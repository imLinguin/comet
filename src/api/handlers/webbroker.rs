use log::{debug, warn};
use protobuf::{Enum, Message};

use super::context::HandlerContext;
use super::error::*;
use crate::proto::common_utils::ProtoPayload;
use crate::proto::galaxy_protocols_webbroker_service::{
    MessageSort, MessageType, SubscribeTopicRequest, SubscribeTopicResponse,
};
use crate::proto::gog_protocols_pb;

pub async fn entry_point(
    payload: &ProtoPayload,
    context: &HandlerContext,
) -> Result<ProtoPayload, MessageHandlingError> {
    debug!("webbroker entry point called");
    let header = &payload.header;

    let message_type: i32 = header.type_().try_into().unwrap();

    if message_type == MessageType::SUBSCRIBE_TOPIC_REQUEST.value() {
        subscribe_topic_request(payload, context).await
    } else {
        warn!(
            "Received unsupported webbroker message type {}",
            message_type
        );
        Err(MessageHandlingError::not_implemented())
    }
}

// Actual handlers of the functions
async fn subscribe_topic_request(
    payload: &ProtoPayload,
    context: &HandlerContext,
) -> Result<ProtoPayload, MessageHandlingError> {
    // This is the stub that just responds with success
    let request_data = SubscribeTopicRequest::parse_from_bytes(&payload.payload);

    let proto = request_data.map_err(MessageHandlingError::proto)?;
    let topic = String::from(proto.topic());

    context.subscribe_topic(topic.clone()).await;
    log::debug!("Webbroker subscribe to {}", topic);
    let mut new_data = SubscribeTopicResponse::new();
    let mut header = gog_protocols_pb::Header::new();
    header.set_sort(MessageSort::MESSAGE_SORT.value().try_into().unwrap());
    header.set_type(
        MessageType::SUBSCRIBE_TOPIC_RESPONSE
            .value()
            .try_into()
            .unwrap(),
    );
    new_data.set_topic(topic);

    let buffer = new_data.write_to_bytes().unwrap();
    header.set_size(buffer.len().try_into().unwrap());
    Ok(ProtoPayload {
        header,
        payload: buffer,
    })
}
