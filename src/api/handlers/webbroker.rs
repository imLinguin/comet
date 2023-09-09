use log::warn;
use protobuf::Message;
use reqwest::Client;

use super::error::*;
use crate::proto::common_utils::ProtoPayload;
use crate::proto::gog_protocols_pb;
use crate::proto::galaxy_protocols_webbroker_service::{
    MessageType, SubscribeTopicRequest, SubscribeTopicResponse, MessageSort,
};

pub async fn entry_point(
    payload: ProtoPayload,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let header = &payload.header;

    let message_type = header.type_();

    if message_type == MessageType::AUTH_REQUEST as u32 {
        Ok(payload)
    } else if message_type == MessageType::SUBSCRIBE_TOPIC_REQUEST as u32 {
        subscribe_topic_request(payload).await
    } else {
        warn!(
            "Recieved unsupported webbroker message type {}",
            message_type
        );
        Err(MessageHandlingError::new(
            MessageHandlingErrorKind::NotImplemented,
        ))
    }
}

// Actual handlers of the functions
async fn subscribe_topic_request(
    payload: ProtoPayload,
) -> Result<ProtoPayload, MessageHandlingError> {
    // This is the stub that just responds with success
    let request_data = SubscribeTopicRequest::parse_from_bytes(&payload.payload);

    let topic = match request_data {
        Ok(proto) => String::from(proto.topic()),
        Err(err) => {
            return Err(MessageHandlingError::new(MessageHandlingErrorKind::Proto(
                err,
            )))
        }
    };

    let mut new_data = SubscribeTopicResponse::new();
    let mut header = gog_protocols_pb::Header::new(); 
    header.set_sort(MessageSort::MESSAGE_SORT as u32);
    header.set_type(MessageType::SUBSCRIBE_TOPIC_RESPONSE as u32);
    new_data.set_topic(topic);

    let buffer = new_data.write_to_bytes().unwrap();
    Ok(ProtoPayload{ header, payload: buffer})
}
