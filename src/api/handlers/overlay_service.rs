use super::{context::HandlerContext, MessageHandlingError, MessageHandlingErrorKind};
use crate::constants;
use crate::proto::common_utils::ProtoPayload;
use crate::proto::{galaxy_protocols_overlay_for_service::*, gog_protocols_pb};
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
        Err(MessageHandlingError::new(MessageHandlingErrorKind::Ignored))
    } else {
        warn!(
            "Received unsupported webbroker message type {}",
            message_type
        );
        Err(MessageHandlingError::new(
            MessageHandlingErrorKind::NotImplemented,
        ))
    }
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
        res.set_access_token(token.refresh_token.clone());
    }
    let payload = res
        .write_to_bytes()
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;
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
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    info!(
        "Overlay notified if it successfully initialized - {}",
        message.initialized_successfully()
    );

    Ok(())
}
