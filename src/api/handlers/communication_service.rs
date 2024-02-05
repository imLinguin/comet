use crate::api::gog;
use crate::api::gog::stats::FieldValue;
use crate::api::handlers::context::HandlerContext;
use crate::api::structs::UserInfo;
use crate::{constants, db};
use log::{debug, info, warn};
use protobuf::{Enum, Message};
use reqwest::{Client, StatusCode};
use std::sync::Arc;
use chrono::Utc;

use crate::proto::common_utils::ProtoPayload;

use super::error::*;
use crate::proto::galaxy_protocols_communication_service::get_user_achievements_response::UserAchievement;
use crate::proto::galaxy_protocols_communication_service::EnvironmentType::ENVIRONMENT_PRODUCTION;
use crate::proto::galaxy_protocols_communication_service::Region::REGION_WORLD_WIDE;
use crate::proto::galaxy_protocols_communication_service::ValueType::{
    VALUE_TYPE_AVGRATE, VALUE_TYPE_FLOAT, VALUE_TYPE_INT,
};
use crate::proto::galaxy_protocols_communication_service::*;
use crate::proto::gog_protocols_pb::Header;

pub async fn entry_point(
    payload: &ProtoPayload,
    context: &mut HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    debug!("Handling in communication service");
    let header = &payload.header;

    let message_type: i32 = header.type_().try_into().unwrap();

    if message_type == MessageType::AUTH_INFO_REQUEST.value() {
        auth_info_request(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::GET_USER_STATS_REQUEST.value() {
        get_user_stats(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::GET_USER_ACHIEVEMENTS_REQUEST.value() {
        get_user_achievements(payload, context, user_info, reqwest_client).await
    } else {
        warn!(
            "Unhandled communication service message type {}",
            message_type
        );
        Err(MessageHandlingError::new(
            MessageHandlingErrorKind::NotImplemented,
        ))
    }
}

async fn auth_info_request(
    payload: &ProtoPayload,
    context: &mut HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request_data = AuthInfoRequest::parse_from_bytes(&payload.payload);
    let request_data = request_data
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let pid = request_data.game_pid();
    // TODO: Use PID to check if process is still running

    let client_id = request_data.client_id();
    let client_secret = request_data.client_secret();
    context.identify_client(client_id, client_secret);
    info!("Client identified as {} {}", client_id, client_secret);

    let token_storage = context.token_store().lock().await;
    let galaxy_token = token_storage
        .get(constants::GALAXY_CLIENT_ID)
        .expect("Failed to get Galaxy token from store");
    let refresh_token = galaxy_token.refresh_token.clone();
    drop(token_storage);

    // Obtain the token (at least attempt to)
    let new_token = gog::users::get_token_for(
        client_id,
        client_secret,
        refresh_token.as_str(),
        reqwest_client,
    )
    .await;

    if let Err(err) = context
        .setup_database(client_id, &user_info.galaxy_user_id)
        .await
    {
        warn!("There was an error setting up the gameplay database, some functionality may be limited {:#?}", err);
    }

    // Use new refresh_token to prepare response
    let mut header = Header::new();
    header.set_type(MessageType::AUTH_INFO_RESPONSE.value().try_into().unwrap());
    header.set_sort(MessageSort::MESSAGE_SORT.value().try_into().unwrap());

    let mut content = AuthInfoResponse::new();
    match new_token {
        Ok(token) => {
            let mut token_storage = context.token_store().lock().await;
            token_storage.insert(String::from(client_id), token.clone());
            drop(token_storage);
            content.set_refresh_token(token.refresh_token);
            context.set_online();
        }
        Err(err) => {
            if let Some(status) = err.status() {
                // user doesn't own the game
                if StatusCode::FORBIDDEN == status {
                    return Err(MessageHandlingError::new(
                        MessageHandlingErrorKind::Unauthorized,
                    ));
                }
            }
        }
    };
    content.set_region(REGION_WORLD_WIDE); // TODO: Handle China region
    content.set_environment_type(ENVIRONMENT_PRODUCTION);
    content.set_user_id(user_info.galaxy_user_id.parse().unwrap());
    content.set_user_name(user_info.username.clone());

    let content_buffer = content.write_to_bytes().unwrap();
    header.set_size(content_buffer.len().try_into().unwrap());

    Ok(ProtoPayload {
        header,
        payload: content_buffer,
    })
}

async fn get_user_stats(
    payload: &ProtoPayload,
    context: &mut HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let has_statistics = db::gameplay::has_statistics(context).await;

    debug!("Statistics in local database: {}", has_statistics);
    let stats: Vec<gog::stats::Stat> = match has_statistics {
        false => gog::stats::fetch_stats(context, &user_info.galaxy_user_id, reqwest_client)
            .await
            .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Network(err)))?,

        true => db::gameplay::get_statistics(context)
            .await
            .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::DB(err)))?,
    };

    if !has_statistics {
        if let Err(err) = db::gameplay::set_statistics(context, &stats).await {
            warn!("Failed to set statistics in gameplay database {:?}", err);
        }
    }

    // Prepare response
    let mut header = Header::new();
    header.set_type(
        MessageType::GET_USER_STATS_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );
    header.set_sort(MessageSort::MESSAGE_SORT.value().try_into().unwrap());

    let mut content = GetUserStatsResponse::new();

    for stat in stats {
        let mut proto_stat = get_user_stats_response::UserStat::new();
        let value_type = match stat.values() {
            FieldValue::INT { .. } => VALUE_TYPE_INT,
            FieldValue::FLOAT { .. } => VALUE_TYPE_FLOAT,
            FieldValue::AVGRATE { .. } => VALUE_TYPE_AVGRATE,
        };
        proto_stat.set_stat_id(stat.stat_id().parse().unwrap());
        proto_stat.set_key(stat.stat_key().to_owned());
        proto_stat.set_value_type(value_type);
        proto_stat.set_increment_only(stat.increment_only().to_owned());
        if let Some(window) = stat.window() {
            proto_stat.set_window_size(window.to_owned());
        }

        match stat.values() {
            FieldValue::INT {
                value,
                default_value,
                min_value,
                max_value,
                max_change,
            } => {
                proto_stat.set_int_value(value.to_owned());
                if let Some(default_value) = default_value {
                    proto_stat.set_int_default_value(default_value.to_owned());
                }
                if let Some(min_value) = min_value {
                    proto_stat.set_int_min_value(min_value.to_owned());
                }
                if let Some(max_value) = max_value {
                    proto_stat.set_int_max_value(max_value.to_owned());
                }
                if let Some(max_change) = max_change {
                    proto_stat.set_int_max_change(max_change.to_owned());
                }
            }
            FieldValue::FLOAT {
                value,
                default_value,
                min_value,
                max_value,
                max_change,
            }
            | FieldValue::AVGRATE {
                value,
                default_value,
                min_value,
                max_value,
                max_change,
            } => {
                proto_stat.set_float_value(value.to_owned());
                if let Some(default_value) = default_value {
                    proto_stat.set_float_default_value(default_value.to_owned());
                }
                if let Some(min_value) = min_value {
                    proto_stat.set_float_min_value(min_value.to_owned());
                }
                if let Some(max_value) = max_value {
                    proto_stat.set_float_max_value(max_value.to_owned());
                }
                if let Some(max_change) = max_change {
                    proto_stat.set_float_max_change(max_change.to_owned());
                }
            }
        }

        content.user_stats.push(proto_stat);
    }
    let content_buffer = content.write_to_bytes().unwrap();
    header.set_size(content_buffer.len().try_into().unwrap());

    Ok(ProtoPayload {
        header,
        payload: content_buffer,
    })
}

async fn get_user_achievements(
    proto_payload: &ProtoPayload,
    mut context: &mut HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let has_achievements = db::gameplay::has_achievements(&mut context).await;

    let (achievements, achievements_mode) = match has_achievements {
        false => gog::achievements::fetch_achievements(
            &context,
            &user_info.galaxy_user_id,
            reqwest_client,
        )
        .await
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Network(err)))?,
        true => db::gameplay::get_achievements(&mut context)
            .await
            .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::DB(err)))?,
    };

    if !has_achievements {
        if let Err(err) =
            db::gameplay::set_achievements(context, &achievements, &achievements_mode).await
        {
            warn!("Failed to set achievements in gameplay database {:?}", err);
        }
    }

    let mut header = Header::new();
    header.set_type(
        MessageType::GET_USER_ACHIEVEMENTS_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );
    header.set_sort(MessageSort::MESSAGE_SORT.value().try_into().unwrap());
    let mut content = GetUserAchievementsResponse::new();
    content.set_achievements_mode(achievements_mode);
    content.set_language("en-US".to_string());

    for achievement in achievements {
        let mut proto_achievement = UserAchievement::new();
        proto_achievement.set_achievement_id(achievement.achievement_id().parse().unwrap());
        proto_achievement.set_key(achievement.achievement_key().to_owned());
        proto_achievement.set_name(achievement.name().to_owned());
        proto_achievement.set_description(achievement.description().to_owned());
        proto_achievement.set_visible_while_locked(achievement.visible().to_owned());
        proto_achievement.set_image_url_locked(achievement.image_url_locked().to_owned());
        proto_achievement.set_image_url_unlocked(achievement.image_url_unlocked().to_owned());
        proto_achievement.set_rarity(achievement.rarity().to_owned());
        proto_achievement
            .set_rarity_level_description(achievement.rarity_level_description().to_owned());
        proto_achievement.set_rarity_level_slug(achievement.rarity_level_slug().to_owned());

        if let Some(date) = achievement.date_unlocked() {
            let parsed_date: chrono::DateTime<Utc> = date.parse().unwrap();
            let timestamp = parsed_date.timestamp() as u32;
            proto_achievement.set_unlock_time(timestamp);
        }
        content.user_achievements.push(proto_achievement);
    }

    let content_buffer = content.write_to_bytes().unwrap();
    header.set_size(content_buffer.len().try_into().unwrap());

    Ok(ProtoPayload {
        header,
        payload: content_buffer,
    })
}
