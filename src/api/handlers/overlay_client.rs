use crate::constants;
use crate::proto::galaxy_protocols_overlay_for_client::*;
use crate::proto::{common_utils::ProtoPayload, gog_protocols_pb};
use log::{error, warn};
use protobuf::{Enum, Message};

use super::{context::HandlerContext, MessageHandlingError, MessageHandlingErrorKind};

pub async fn entry_point(
    payload: &ProtoPayload,
    context: &HandlerContext,
) -> Result<ProtoPayload, MessageHandlingError> {
    let header = &payload.header;

    let message_type: i32 = header.type_().try_into().unwrap();

    if message_type == MessageType::OVERLAY_FRONTEND_INIT_DATA_REQUEST.value() {
        overlay_data_request(payload, context).await
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

async fn overlay_data_request(
    payload: &ProtoPayload,
    context: &HandlerContext,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request = OverlayFrontendInitDataRequest::parse_from_bytes(&payload.payload)
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let mut res = OverlayFrontendInitDataResponse::new();
    res.set_data(constants::GALAXY_INIT_DATA.to_owned());
    let payload = res
        .write_to_bytes()
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let mut header = gog_protocols_pb::Header::new();
    header.set_sort(MessageSort::MESSAGE_SORT.value().try_into().unwrap());
    header.set_type(
        MessageType::OVERLAY_FRONTEND_INIT_DATA_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );
    header.set_size(payload.len().try_into().unwrap());

    Ok(ProtoPayload { header, payload })
}
