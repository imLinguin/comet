use crate::api::gog;
use crate::api::structs::UserInfo;
use crate::constants;
use log::{debug, info, warn};
use protobuf::{Enum, Message};
use reqwest::Client;
use std::sync::Arc;

use crate::constants::TokenStorage;
use crate::proto::common_utils::ProtoPayload;

use super::error::*;
use crate::proto::galaxy_protocols_communication_service::EnvironmentType::ENVIRONMENT_PRODUCTION;
use crate::proto::galaxy_protocols_communication_service::Region::REGION_WORLD_WIDE;
use crate::proto::galaxy_protocols_communication_service::*;
use crate::proto::gog_protocols_pb::Header;

pub async fn entry_point(
    payload: &ProtoPayload,
    token_store: &TokenStorage,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    debug!("Handling in communication service");
    let header = &payload.header;

    let message_type: i32 = header.type_().try_into().unwrap();

    if message_type == MessageType::AUTH_INFO_REQUEST.value() {
        auth_info_request(payload, token_store, user_info, reqwest_client).await
    } else if message_type == MessageType::GET_USER_STATS_REQUEST.value() {
        get_user_stats(payload, token_store, user_info, reqwest_client).await
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
    token_store: &TokenStorage,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    let request_data = AuthInfoRequest::parse_from_bytes(&payload.payload);
    let request_data = request_data
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::Proto(err)))?;

    let pid = request_data.game_pid();
    // TODO: Decide whether the process is trusted

    let client_id = request_data.client_id();
    let client_secret = request_data.client_secret();
    info!("Client identified as {} {}", client_id, client_secret);

    let token_storage = token_store.lock().await;
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
    .await
    .expect("Failed to obtain the token, game will run offline");

    let mut token_storage = token_store.lock().await;
    token_storage.insert(String::from(client_id), new_token.clone());
    drop(token_storage);

    // Use new refresh_token to prepare response
    let mut header = Header::new();
    header.set_type(MessageType::AUTH_INFO_RESPONSE.value().try_into().unwrap());
    header.set_sort(MessageSort::MESSAGE_SORT.value().try_into().unwrap());

    let mut content = AuthInfoResponse::new();
    content.set_refresh_token(new_token.refresh_token);
    content.set_region(REGION_WORLD_WIDE);
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
    token_store: &TokenStorage,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<ProtoPayload, MessageHandlingError> {
    todo!();
}
