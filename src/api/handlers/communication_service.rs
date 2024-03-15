use crate::api::gog;
use crate::api::gog::stats::FieldValue;
use crate::api::handlers::context::HandlerContext;
use crate::api::structs::{DataSource, UserInfo};
use crate::db::gameplay::{set_stat_float, set_stat_int};
use crate::{constants, db};
use chrono::{Local, TimeZone, Utc};
use log::{debug, info, warn};
use protobuf::{Enum, Message};
use reqwest::{Client, StatusCode};
use std::sync::Arc;

use crate::proto::common_utils::ProtoPayload;

use super::error::*;
use crate::proto::galaxy_protocols_communication_service::get_user_achievements_response::UserAchievement;
use crate::proto::galaxy_protocols_communication_service::EnvironmentType::ENVIRONMENT_PRODUCTION;
use crate::proto::galaxy_protocols_communication_service::Region::REGION_WORLD_WIDE;
use crate::proto::galaxy_protocols_communication_service::ValueType::{
    VALUE_TYPE_AVGRATE, VALUE_TYPE_FLOAT, VALUE_TYPE_INT, VALUE_TYPE_UNDEFINED,
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
    } else if message_type == MessageType::UPDATE_USER_STAT_REQUEST.value() {
        update_user_stat(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::GET_USER_ACHIEVEMENTS_REQUEST.value() {
        get_user_achievements(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::UNLOCK_USER_ACHIEVEMENT_REQUEST.value() {
        unlock_user_achievement(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::CLEAR_USER_ACHIEVEMENT_REQUEST.value() {
        clear_user_achievement(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::GET_LEADERBOARDS_REQUEST.value() {
        get_leaderboards(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::GET_LEADERBOARDS_BY_KEY_REQUEST.value() {
        get_leaderboards_by_key(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::GET_LEADERBOARD_ENTRIES_GLOBAL_REQUEST.value() {
        get_leaderboard_entries_global(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::GET_LEADERBOARD_ENTRIES_AROUND_USER_REQUEST.value() {
        get_leaderboard_entries_around_user(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::GET_LEADERBOARD_ENTRIES_FOR_USERS_REQUEST.value() {
        get_leaderboard_entries_for_users(payload, context, user_info, reqwest_client).await
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

    let _pid = request_data.game_pid();
    // TODO: Use PID to connect to overlay

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
        panic!(
            "There was an error setting up the gameplay database {:#?}",
            err
        );
    }

    // Use new refresh_token to prepare response
    let mut header = Header::new();
    header.set_type(MessageType::AUTH_INFO_RESPONSE.value().try_into().unwrap());

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
            warn!("There was an error getting the access token");
            if let Some(status) = err.status() {
                // user doesn't own the game
                if StatusCode::FORBIDDEN == status {
                    return Err(MessageHandlingError::new(
                        MessageHandlingErrorKind::Unauthorized,
                    ));
                }
            }
            // Check if we can continue offline
            let ach = db::gameplay::has_achievements(context).await;
            let stat = db::gameplay::has_statistics(context).await;
            if !stat && !ach {
                panic!("No statistics or achievements locally, can't continue");
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
    _payload: &ProtoPayload,
    context: &HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let new_stats =
        gog::stats::fetch_stats(context, &user_info.galaxy_user_id, reqwest_client).await;
    let db_stats = db::gameplay::get_statistics(context, false).await;

    let mut stats_source = DataSource::Online;

    let stats = match new_stats {
        Ok(stats) => stats,
        Err(_) => match db_stats {
            Ok(stats) => {
                stats_source = DataSource::Local;
                stats
            }
            Err(_) => panic!("Unable to retrieve stats"),
        },
    };

    if stats_source == DataSource::Online {
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

    let mut content = GetUserStatsResponse::new();

    for stat in stats {
        let mut proto_stat = get_user_stats_response::UserStat::new();
        let value_type = match stat.values() {
            FieldValue::Int { .. } => VALUE_TYPE_INT,
            FieldValue::Float { .. } => VALUE_TYPE_FLOAT,
            FieldValue::Avgrate { .. } => VALUE_TYPE_AVGRATE,
        };
        proto_stat.set_stat_id(stat.stat_id().parse().unwrap());
        proto_stat.set_key(stat.stat_key().to_owned());
        proto_stat.set_value_type(value_type);
        proto_stat.set_increment_only(stat.increment_only().to_owned());
        if let Some(window) = stat.window() {
            proto_stat.set_window_size(window.to_owned());
        }

        match stat.values() {
            FieldValue::Int {
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
            FieldValue::Float {
                value,
                default_value,
                min_value,
                max_value,
                max_change,
            }
            | FieldValue::Avgrate {
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

async fn update_user_stat(
    proto_payload: &ProtoPayload,
    context: &mut HandlerContext,
    _user_info: Arc<UserInfo>,
    _reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request_data = UpdateUserStatRequest::parse_from_bytes(&proto_payload.payload);
    let request_data = request_data
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let stat_id: u64 = request_data.stat_id();
    let stat_id: i64 = stat_id.try_into().unwrap();
    let value_type = request_data.value_type();
    match value_type {
        VALUE_TYPE_FLOAT | VALUE_TYPE_AVGRATE => {
            let value = request_data.float_value();
            set_stat_float(context, stat_id, value)
                .await
                .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::DB(err)))?;
        }
        VALUE_TYPE_INT => {
            let value = request_data.int_value();
            set_stat_int(context, stat_id, value)
                .await
                .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::DB(err)))?;
        }
        VALUE_TYPE_UNDEFINED => {
            warn!("Undefined value type, ignoring");
        }
    };

    context.set_updated_stats(true);

    let mut header = Header::new();
    header.set_type(
        MessageType::UPDATE_USER_STAT_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );

    Ok(ProtoPayload {
        header,
        payload: Vec::new(),
    })
}

async fn get_user_achievements(
    _proto_payload: &ProtoPayload,
    context: &HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let online_achievements =
        gog::achievements::fetch_achievements(context, &user_info.galaxy_user_id, reqwest_client)
            .await;
    let local_achievements = db::gameplay::get_achievements(context, false).await;

    let mut achievements_source = DataSource::Online;
    let (achievements, achievements_mode) = match online_achievements {
        Ok(achievements) => achievements,
        Err(_) => match local_achievements {
            Ok(achievements) => {
                achievements_source = DataSource::Local;
                achievements
            }
            Err(_) => panic!("Unable to load achievements"),
        },
    };

    if achievements_source == DataSource::Online {
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

async fn unlock_user_achievement(
    proto_payload: &ProtoPayload,
    context: &mut HandlerContext,
    _user_info: Arc<UserInfo>,
    _reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request_data = UnlockUserAchievementRequest::parse_from_bytes(&proto_payload.payload);
    let request_data = request_data
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let ach_id: i64 = request_data.achievement_id().try_into().unwrap();
    let timestamp = request_data.time();
    let dt = Local.timestamp_opt(timestamp.into(), 0).unwrap();
    let timestamp_string = Some(dt.to_utc().to_rfc3339());

    // FIXME: Handle errors gracefully
    // Check with database first
    let achievement = db::gameplay::get_achievement(context, ach_id)
        .await
        .expect("Failed to read database");

    if achievement.date_unlocked().is_none() {
        db::gameplay::set_achievement(context, ach_id, timestamp_string.clone())
            .await
            .expect("Failed to write achievement to database");
        context.set_updated_achievements(true);
    }

    let mut header = Header::new();
    header.set_type(
        MessageType::UNLOCK_USER_ACHIEVEMENT_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );

    Ok(ProtoPayload {
        header,
        payload: Vec::new(),
    })
}

async fn clear_user_achievement(
    proto_payload: &ProtoPayload,
    context: &mut HandlerContext,
    _user_info: Arc<UserInfo>,
    _reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request_data = ClearUserAchievementRequest::parse_from_bytes(&proto_payload.payload);
    let request_data = request_data
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let ach_id: i64 = request_data.achievement_id().try_into().unwrap();

    db::gameplay::set_achievement(context, ach_id, None)
        .await
        .expect("Failed to write achievement to database");
    context.set_updated_achievements(true);

    let mut header = Header::new();
    header.set_type(
        MessageType::CLEAR_USER_ACHIEVEMENT_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );

    Ok(ProtoPayload {
        header,
        payload: Vec::new(),
    })
}
async fn get_leaderboards(
    _proto_payload: &ProtoPayload,
    context: &mut HandlerContext,
    _user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    super::utils::handle_leaderboards_query(context, reqwest_client, [] as [(&str, &str); 0]).await
}

async fn get_leaderboards_by_key(
    proto_payload: &ProtoPayload,
    context: &mut HandlerContext,
    _user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request = GetLeaderboardsByKeyRequest::parse_from_bytes(&proto_payload.payload)
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let keys = request.key.join(",");
    super::utils::handle_leaderboards_query(context, reqwest_client, [("keys", keys)]).await
}

async fn get_leaderboard_entries_global(
    proto_payload: &ProtoPayload,
    context: &mut HandlerContext,
    _user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request = GetLeaderboardEntriesGlobalRequest::parse_from_bytes(&proto_payload.payload)
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let params = [
        ("range_start", request.range_start().to_string()),
        ("range_end", request.range_end().to_string()),
    ];

    Ok(super::utils::handle_leaderboard_entries_request(
        context,
        reqwest_client,
        request.leaderboard_id(),
        params,
    )
    .await)
}
async fn get_leaderboard_entries_around_user(
    proto_payload: &ProtoPayload,
    context: &mut HandlerContext,
    _user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request = GetLeaderboardEntriesAroundUserRequest::parse_from_bytes(&proto_payload.payload)
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let user_id = request.user_id() << 8 >> 8;

    let params = [
        ("count_before", request.count_before().to_string()),
        ("count_after", request.count_after().to_string()),
        ("user", user_id.to_string()),
    ];

    Ok(super::utils::handle_leaderboard_entries_request(
        context,
        reqwest_client,
        request.leaderboard_id(),
        params,
    )
    .await)
}

async fn get_leaderboard_entries_for_users(
    proto_payload: &ProtoPayload,
    context: &mut HandlerContext,
    _user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request = GetLeaderboardEntriesForUsersRequest::parse_from_bytes(&proto_payload.payload)
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let user_ids: String = request
        .user_ids
        .iter()
        .map(|id| (id << 8 >> 8).to_string())
        .collect::<Vec<String>>()
        .join(",");

    let params = [("users", user_ids)];

    Ok(super::utils::handle_leaderboard_entries_request(
        context,
        reqwest_client,
        request.leaderboard_id(),
        params,
    )
    .await)
}
