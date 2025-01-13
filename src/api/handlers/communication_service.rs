use crate::api::gog;
use crate::api::gog::stats::FieldValue;
use crate::api::handlers::context::HandlerContext;
use crate::api::structs::{DataSource, IDType, UserInfo};
use crate::db::gameplay::{set_stat_float, set_stat_int};
use crate::paths::REDISTS_STORAGE;
use crate::{constants, db};
use base64::prelude::*;
use chrono::{TimeZone, Utc};
use log::{debug, info, warn};
use protobuf::{Enum, Message};
use reqwest::{Client, StatusCode};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

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
    context: &HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    debug!("Handling in communication service");
    let header = &payload.header;

    let message_type: i32 = header.type_().try_into().unwrap();

    if message_type == MessageType::LIBRARY_INFO_REQUEST.value() {
        library_info_request(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::AUTH_INFO_REQUEST.value() {
        auth_info_request(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::GET_USER_STATS_REQUEST.value() {
        get_user_stats(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::UPDATE_USER_STAT_REQUEST.value() {
        update_user_stat(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::DELETE_USER_STATS_REQUEST.value() {
        delete_user_stats(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::GET_USER_ACHIEVEMENTS_REQUEST.value() {
        get_user_achievements(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::UNLOCK_USER_ACHIEVEMENT_REQUEST.value() {
        unlock_user_achievement(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::CLEAR_USER_ACHIEVEMENT_REQUEST.value() {
        clear_user_achievement(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::DELETE_USER_ACHIEVEMENTS_REQUEST.value() {
        delete_user_achievements(payload, context, user_info, reqwest_client).await
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
    } else if message_type == MessageType::SET_LEADERBOARD_SCORE_REQUEST.value() {
        set_leaderboard_score(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::CREATE_LEADERBOARD_REQUEST.value() {
        create_leaderboard(payload, context, user_info, reqwest_client).await
    } else if message_type == MessageType::START_GAME_SESSION_REQUEST.value() {
        start_game_session(payload, context, user_info, reqwest_client).await
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

async fn library_info_request(
    payload: &ProtoPayload,
    _context: &HandlerContext,
    _user_info: Arc<UserInfo>,
    _reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request_data = LibraryInfoRequest::parse_from_bytes(&payload.payload);
    let request_data = request_data
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let compiler_type = request_data.compiler_type();
    let compiler_version = request_data.compiler_version();

    log::debug!("Compiler {:?} Version: {}", compiler_type, compiler_version);
    let path = match compiler_type {
        CompilerType::COMPILER_TYPE_MSVC => {
            REDISTS_STORAGE.join(format!("peer/msvc-{}", compiler_version))
        }
        #[cfg(target_os = "macos")]
        CompilerType::COMPILER_TYPE_CLANG => REDISTS_STORAGE.join("peer"),
        _ => REDISTS_STORAGE.join("peer/msvc-18"),
    };
    let path_str = path.to_str().unwrap().to_string();

    let mut header = Header::new();
    header.set_type(
        MessageType::LIBRARY_INFO_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );

    let mut data = LibraryInfoResponse::new();
    data.set_location(path_str);
    data.set_update_status(UpdateStatus::UPDATE_COMPLETE);

    let payload = data.write_to_bytes().unwrap();
    header.set_size(payload.len().try_into().unwrap());

    Ok(ProtoPayload { header, payload })
}

async fn auth_info_request(
    payload: &ProtoPayload,
    context: &HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request_data = AuthInfoRequest::parse_from_bytes(&payload.payload);
    let request_data = request_data
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let client_id = request_data.client_id();
    let client_secret = request_data.client_secret();
    let openid = request_data.openid();

    if !context.client_identified().await {
        context.identify_client(client_id, client_secret).await;
        info!("Client identified as {} {}", client_id, client_secret);
    }

    let pid = request_data.game_pid();
    info!("Game PID: {}", pid);

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
        openid,
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

    {
        let mut data = OverlayStateChangeNotification::new();
        data.set_overlay_state(
            overlay_state_change_notification::OverlayState::OVERLAY_STATE_INITIALIZED,
        );
        let data_buf = data.write_to_bytes().unwrap();
        let mut header = Header::new();
        header.set_sort(1);
        header.set_type(
            MessageType::OVERLAY_STATE_CHANGE_NOTIFICATION
                .value()
                .try_into()
                .unwrap(),
        );
        header.set_size(data_buf.len().try_into().unwrap());
        let header_buf = header.write_to_bytes().unwrap();
        let header_size: u16 = header_buf.len().try_into().unwrap();

        let mut buffer = vec![];
        buffer.extend(header_size.to_be_bytes());
        buffer.extend(header_buf);
        buffer.extend(data_buf);
        let _ = context.socket_mut().await.write_all(&buffer).await;
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
            context.set_online().await;
        }
        Err(err) => {
            warn!("There was an error getting the access token {:?}", err);
            if let Some(status) = err.status() {
                // user doesn't own the game
                if StatusCode::FORBIDDEN == status {
                    return Err(MessageHandlingError::new(
                        MessageHandlingErrorKind::Unauthorized,
                    ));
                }
            }
            // Check if we can continue offline
            let db_connection = context.db_connection().await;
            let ach = db::gameplay::has_achievements(&db_connection).await;
            let stat = db::gameplay::has_statistics(&db_connection).await;
            if !stat && !ach {
                panic!("No statistics or achievements locally, can't continue");
            }
        }
    };
    content.set_region(REGION_WORLD_WIDE); // TODO: Handle China region
    content.set_environment_type(ENVIRONMENT_PRODUCTION);
    let user_id = IDType::User(user_info.galaxy_user_id.parse().unwrap());
    content.set_user_id(user_id.value());
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
    let new_stats = gog::stats::fetch_stats(
        context.token_store(),
        &context.client_id().await.unwrap(),
        &user_info.galaxy_user_id,
        reqwest_client,
    )
    .await;
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
        if let Err(err) = db::gameplay::set_statistics(context.db_connection().await, &stats).await
        {
            warn!("Failed to set statistics in gameplay database {:?}", err);
        }
    }

    context.set_updated_stats(true).await;

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
    context: &HandlerContext,
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

    context.set_updated_stats(true).await;

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

async fn delete_user_stats(
    _proto_payload: &ProtoPayload,
    context: &HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    gog::stats::delete_stats(context, reqwest_client, &user_info.galaxy_user_id)
        .await
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Network(err)))?;

    db::gameplay::reset_stats(context)
        .await
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::DB(err)))?;

    let mut header = Header::new();
    header.set_type(
        MessageType::DELETE_USER_STATS_RESPONSE
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
    let client_id = context.client_id().await.unwrap();
    let online_achievements = gog::achievements::fetch_achievements(
        context.token_store(),
        &client_id,
        &user_info.galaxy_user_id,
        reqwest_client,
    )
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
        if let Err(err) = db::gameplay::set_achievements(
            context.db_connection().await,
            &achievements,
            &achievements_mode,
        )
        .await
        {
            warn!("Failed to set achievements in gameplay database {:?}", err);
        }
        context.set_updated_achievements(true).await;
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
    content.set_language(crate::LOCALE.clone());

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
    context: &HandlerContext,
    _user_info: Arc<UserInfo>,
    _reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request_data = UnlockUserAchievementRequest::parse_from_bytes(&proto_payload.payload);
    let request_data = request_data
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let ach_id: i64 = request_data.achievement_id().try_into().unwrap();
    let timestamp = request_data.time();
    let dt = Utc.timestamp_opt(timestamp.into(), 0).unwrap();
    let timestamp_string = Some(dt.to_rfc3339().to_string());

    // FIXME: Handle errors gracefully
    // Check with database first
    let mut achievement = db::gameplay::get_achievement(context, ach_id)
        .await
        .expect("Failed to read database");

    if achievement.date_unlocked().is_none() {
        info!(
            "Unlocking achievement {}, {}",
            achievement.achievement_key(),
            achievement.name()
        );
        db::gameplay::set_achievement(context, ach_id, timestamp_string.clone())
            .await
            .expect("Failed to write achievement to database");
        context.set_updated_achievements(true).await;
        achievement.date_unlocked = timestamp_string;
        let _ = context.achievement_sender().send(achievement);
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
    context: &HandlerContext,
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
    context.set_updated_achievements(true).await;

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

async fn delete_user_achievements(
    _proto_payload: &ProtoPayload,
    context: &HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    gog::achievements::delete_achievements(context, reqwest_client, &user_info.galaxy_user_id)
        .await
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Network(err)))?;

    db::gameplay::reset_achievements(context)
        .await
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::DB(err)))?;

    let mut header = Header::new();
    header.set_type(
        MessageType::DELETE_USER_ACHIEVEMENTS_RESPONSE
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
    context: &HandlerContext,
    _user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    super::utils::handle_leaderboards_query(context, reqwest_client, [] as [(&str, &str); 0]).await
}

async fn get_leaderboards_by_key(
    proto_payload: &ProtoPayload,
    context: &HandlerContext,
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
    context: &HandlerContext,
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
    context: &HandlerContext,
    _user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request = GetLeaderboardEntriesAroundUserRequest::parse_from_bytes(&proto_payload.payload)
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let user_id = IDType::parse(request.user_id());

    let params = [
        ("count_before", request.count_before().to_string()),
        ("count_after", request.count_after().to_string()),
        ("user", user_id.inner().to_string()),
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
    context: &HandlerContext,
    _user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request = GetLeaderboardEntriesForUsersRequest::parse_from_bytes(&proto_payload.payload)
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let user_ids: String = request
        .user_ids
        .iter()
        .map(|id| IDType::parse(*id).inner().to_string())
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

async fn set_leaderboard_score(
    proto_payload: &ProtoPayload,
    context: &HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request = SetLeaderboardScoreRequest::parse_from_bytes(&proto_payload.payload)
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let id = request.leaderboard_id().to_string();
    info!(
        "Got leaderboard score request for {}, score = {}",
        id,
        request.score()
    );
    let current_score = match db::gameplay::get_leaderboard_score(context, &id).await {
        Ok((score, _old_rank, _entry_total_count, _force, _details)) => score,
        Err(sqlx::Error::RowNotFound) => 0,
        Err(err) => return Err(MessageHandlingError::new(MessageHandlingErrorKind::DB(err))),
    };
    let mut header = Header::new();
    header.set_type(
        MessageType::SET_LEADERBOARD_SCORE_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );
    if !request.force_update() && (request.score() <= current_score) {
        log::warn!("No improvement {} vs {}", request.score(), current_score);
        header
            .mut_special_fields()
            .mut_unknown_fields()
            .add_varint(101, 409);

        return Ok(ProtoPayload {
            header,
            payload: Vec::new(),
        });
    }
    let details = request.details();
    let details = BASE64_URL_SAFE_NO_PAD.encode(details);
    let post_details = if details.is_empty() {
        None
    } else {
        Some(details.clone())
    };
    let response = gog::leaderboards::post_leaderboard_score(
        context,
        reqwest_client,
        &user_info.galaxy_user_id,
        request.leaderboard_id().try_into().unwrap(),
        request.score(),
        request.force_update(),
        post_details,
    )
    .await;

    match response {
        Ok(data) => {
            db::gameplay::set_leaderboard_score(
                context,
                &id,
                request.score(),
                request.force_update(),
                &details,
            )
            .await
            .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::DB(err)))?;
            db::gameplay::set_leaderboard_rank(
                context,
                &id,
                data.new_rank,
                data.leaderboard_entry_total_count,
            )
            .await
            .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::DB(err)))?;

            let mut proto_data = SetLeaderboardScoreResponse::new();
            proto_data.set_score(request.score());
            proto_data.set_old_rank(data.old_rank);
            proto_data.set_new_rank(data.new_rank);
            proto_data.set_leaderboard_entry_total_count(data.leaderboard_entry_total_count);
            let payload = proto_data
                .write_to_bytes()
                .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;
            header.set_size(payload.len().try_into().unwrap());
            return Ok(ProtoPayload { header, payload });
        }
        Err(err) => {
            log::error!("Failed to set leaderboard score, {:?}", err);

            db::gameplay::set_leaderboard_score(
                context,
                &id,
                request.score(),
                request.force_update(),
                &details,
            )
            .await
            .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::DB(err)))?;
            if err.status().is_none() || err.status().is_some_and(|s| s.as_u16() != 409) {
                db::gameplay::set_leaderboad_changed(context, &id, true)
                    .await
                    .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::DB(err)))?;
                context.set_updated_leaderboards(true).await;
            }
        }
    }

    let (_score, old_rank, entry_total_count, _force, _details) =
        db::gameplay::get_leaderboard_score(context, &id)
            .await
            .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::DB(err)))?;
    let new_rank = if old_rank != 0 { old_rank } else { 1 };
    let entry_total_count = if old_rank != 0 {
        entry_total_count
    } else {
        entry_total_count + 1
    };

    let mut proto_data = SetLeaderboardScoreResponse::new();
    proto_data.set_score(request.score());
    proto_data.set_old_rank(old_rank);
    proto_data.set_new_rank(new_rank);
    proto_data.set_leaderboard_entry_total_count(entry_total_count);
    let payload = proto_data
        .write_to_bytes()
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;
    header.set_size(payload.len().try_into().unwrap());
    Ok(ProtoPayload { header, payload })
}

async fn create_leaderboard(
    proto_payload: &ProtoPayload,
    context: &HandlerContext,
    _user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let mut request = CreateLeaderboardRequest::parse_from_bytes(&proto_payload.payload)
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let key = request.take_key();
    let name = request.take_name();
    let sort_method = match request.sort_method() {
        SortMethod::SORT_METHOD_ASCENDING => "asc",
        SortMethod::SORT_METHOD_DESCENDING => "desc",
        _ => unreachable!("Client should define valid sort method"),
    }
    .to_string();
    let display_type = match request.display_type() {
        DisplayType::DISPLAY_TYPE_NUMERIC => "numeric",
        DisplayType::DISPLAY_TYPE_TIME_SECONDS => "seconds",
        DisplayType::DISPLAY_TYPE_TIME_MILLISECONDS => "milliseconds",
        DisplayType::DISPLAY_TYPE_UNDEFINED => {
            unreachable!("Client should define a valid display type")
        }
    }
    .to_string();

    let leaderboard_id = gog::leaderboards::create_leaderboard(
        context,
        reqwest_client,
        key,
        name,
        sort_method,
        display_type,
    )
    .await
    .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Network(err)))?;

    let mut response = CreateLeaderboardResponse::new();
    response.set_leaderboard_id(leaderboard_id.parse().unwrap());

    let mut header = Header::new();
    header.set_type(
        MessageType::CREATE_LEADERBOARD_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );

    let payload = response
        .write_to_bytes()
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;
    header.set_size(payload.len().try_into().unwrap());

    Ok(ProtoPayload { header, payload })
}

async fn start_game_session(
    proto_payload: &ProtoPayload,
    context: &HandlerContext,
    _user_info: Arc<UserInfo>,
    _reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request = StartGameSessionRequest::parse_from_bytes(&proto_payload.payload)
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    info!("Requested session start for {}", request.game_pid());

    #[cfg(unix)]
    {
        let pipe_name = format!(
            "/tmp/Galaxy-{}-CommunicationService-Overlay",
            request.game_pid()
        );
        context.register_overlay_listener(pipe_name).await;
    }
    #[cfg(windows)]
    {
        let pipe_name = format!(
            r"\\.\pipe\Galaxy-{}-CommunicationService-Overlay",
            request.game_pid()
        );
        context.register_overlay_listener(pipe_name).await;
    }
    let mut header = Header::new();
    header.set_type(
        MessageType::START_GAME_SESSION_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );

    Ok(ProtoPayload {
        header,
        payload: vec![],
    })
}
