use super::{context::HandlerContext, MessageHandlingError};
use crate::api::gog::achievements::Achievement;
use crate::constants;
use crate::proto::common_utils::ProtoPayload;
use crate::proto::{galaxy_protocols_overlay_for_service::*, gog_protocols_pb};
use chrono::Utc;
use log::{debug, info, warn};
use protobuf::{Enum, Message};

pub async fn entry_point(
    payload: &ProtoPayload,
    context: &HandlerContext,
) -> Result<ProtoPayload, MessageHandlingError> {
    debug!("overlay <-> service entry point called");
    let header = &payload.header;
    let message_type: i32 = header.type_().try_into().unwrap();

    if message_type == MessageType::ACCESS_TOKEN_REQUEST.value() {
        access_token(payload, context).await
    } else if message_type == MessageType::OVERLAY_INITIALIZATION_NOTIFICATION.value() {
        init_notification(payload).await?;
        Err(MessageHandlingError::ignored())
    } else {
        warn!(
            "Received unsupported ov_service message type {}",
            message_type
        );
        Err(MessageHandlingError::not_implemented())
    }
}

pub async fn achievement_notification(
    achievement: Achievement,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut res_data = NotifyAchievementUnlocked::new();
    res_data.set_key(achievement.achievement_key);
    res_data.set_name(achievement.name);
    res_data.set_description(achievement.description);
    res_data.set_achievement_id(achievement.achievement_id.parse().unwrap());
    if let Some(date) = achievement.date_unlocked {
        let parsed_date: chrono::DateTime<Utc> = date.parse().unwrap();
        let timestamp = parsed_date.timestamp() as u64;
        res_data.set_unlock_time(timestamp);
    }
    res_data.set_image_url_locked(achievement.image_url_locked);
    res_data.set_image_url_unlocked(achievement.image_url_unlocked);
    res_data.set_visible_while_locked(achievement.visible);
    let res_buf = res_data.write_to_bytes()?;

    let mut header = gog_protocols_pb::Header::new();
    header.set_sort(MessageSort::MESSAGE_SORT.value().try_into()?);
    header.set_type(
        MessageType::NOTIFY_ACHIEVEMENT_UNLOCKED
            .value()
            .try_into()
            .unwrap(),
    );
    header.set_size(res_buf.len().try_into()?);
    let header_buffer = header.write_to_bytes()?;
    let header_size: u16 = header_buffer.len().try_into().unwrap();
    let header_buf = header_size.to_be_bytes();

    let mut message_buffer = Vec::with_capacity(2 + header_buffer.len() + res_buf.len());
    message_buffer.extend(header_buf);
    message_buffer.extend(header_buffer);
    message_buffer.extend(res_buf);

    Ok(message_buffer)
}

async fn access_token(
    _payload: &ProtoPayload,
    context: &HandlerContext,
) -> Result<ProtoPayload, MessageHandlingError> {
    let tokens = context.token_store();

    let galaxy_access_token = {
        let tokens = tokens.lock().await;
        tokens.get(constants::GALAXY_CLIENT_ID).cloned()
    };

    let mut res = AccessTokenResponse::new();
    if let Some(token) = galaxy_access_token {
        res.set_access_token(token.access_token.clone());
    }
    let payload = res.write_to_bytes().map_err(MessageHandlingError::proto)?;
    let mut header = gog_protocols_pb::Header::new();
    header.set_sort(MessageSort::MESSAGE_SORT.value().try_into().unwrap());
    header.set_type(
        MessageType::ACCESS_TOKEN_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );
    header.set_size(payload.len().try_into().unwrap());

    Ok(ProtoPayload { header, payload })
}

async fn init_notification(payload: &ProtoPayload) -> Result<(), MessageHandlingError> {
    let message = OverlayInitializationNotification::parse_from_bytes(&payload.payload)
        .map_err(MessageHandlingError::proto)?;

    info!(
        "Overlay notified if it successfully initialized - {}",
        message.initialized_successfully()
    );

    Ok(())
}
