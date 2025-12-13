use crate::proto::common_utils::ProtoPayload;
use crate::proto::gog_protocols_pb::Header;
use crate::{api::gog::overlay::OverlayPeerMessage, proto::galaxy_protocols_overlay_for_peer::*};
use log::warn;
use protobuf::{Enum, Message};

use super::{context::HandlerContext, MessageHandlingError};

pub async fn entry_point(
    payload: &ProtoPayload,
    context: &HandlerContext,
) -> Result<ProtoPayload, MessageHandlingError> {
    let header = &payload.header;

    let message_type: i32 = header.type_().try_into().unwrap();

    if message_type == MessageType::SHOW_WEB_PAGE.value() {
        let _ = show_web_page(payload, context).await;
    } else if message_type == MessageType::VISIBILITY_CHANGE_NOTIFICATION.value() {
        let _ = visibility_change(payload, context).await;
    } else if message_type == MessageType::SHOW_INVITATION_DIALOG.value() {
        let _ = show_invitation(payload, context).await;
    } else if message_type == MessageType::GAME_JOIN_REQUEST_NOTIFICATION.value() {
        let _ = game_join(payload, context).await;
    } else if message_type == MessageType::OVERLAY_INITIALIZED_NOTIFICATION.value() {
        let _ = overlay_initialized(payload, context).await;
    } else {
        warn!("Received unsupported peer message type {}", message_type);
        return Err(MessageHandlingError::not_implemented());
    }
    Err(MessageHandlingError::ignored())
}

async fn show_web_page(
    payload: &ProtoPayload,
    context: &HandlerContext,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request = ShowWebPage::parse_from_bytes(&payload.payload)?;
    let pid = context.get_pid().await;
    let msg = OverlayPeerMessage::OpenWebPage(request.url().to_owned());
    let _ = context.overlay_sender().send((pid, msg));
    Ok(())
}

async fn visibility_change(
    payload: &ProtoPayload,
    context: &HandlerContext,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request = VisibilityChangeNotification::parse_from_bytes(&payload.payload)?;
    let pid = context.get_pid().await;
    let msg = OverlayPeerMessage::VisibilityChange(request.visible());
    let _ = context.overlay_sender().send((pid, msg));
    Ok(())
}

async fn show_invitation(
    payload: &ProtoPayload,
    context: &HandlerContext,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request = ShowInvitationDialog::parse_from_bytes(&payload.payload)?;
    let pid = context.get_pid().await;
    let msg = OverlayPeerMessage::InvitationDialog(request.connection_string().to_owned());
    let _ = context.overlay_sender().send((pid, msg));
    Ok(())
}

async fn game_join(
    payload: &ProtoPayload,
    context: &HandlerContext,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request = GameJoinRequestNotification::parse_from_bytes(&payload.payload)?;
    let pid = context.get_pid().await;
    let msg = OverlayPeerMessage::GameJoin((
        request.inviter_id(),
        request.client_id().to_owned(),
        request.connection_string().to_owned(),
    ));
    let _ = context.overlay_sender().send((pid, msg));
    Ok(())
}

async fn overlay_initialized(
    payload: &ProtoPayload,
    context: &HandlerContext,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pid = context.get_pid().await;
    let msg = OverlayPeerMessage::DisablePopups(payload.payload.clone());
    let _ = context.overlay_sender().send((pid, msg));
    Ok(())
}

// ==================================================
// Functions used by overlay thread to send messages
// ==================================================

async fn encode_message(
    msg_type: u32,
    msg_data: Vec<u8>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut header = Header::new();
    header.set_sort(MessageSort::MESSAGE_SORT.value() as u32);
    header.set_type(msg_type);
    header.set_size(msg_data.len().try_into()?);
    header.set_oseq(rand::random());
    let header_bytes = header.write_to_bytes()?;
    let header_len: u16 = header_bytes.len().try_into()?;
    let bytes = header_len.to_be_bytes();

    let mut res = Vec::with_capacity(2 + header_bytes.len() + msg_data.len());
    res.extend(bytes);
    res.extend(header_bytes);
    res.extend(msg_data);

    Ok(res)
}

pub async fn encode_open_web_page(
    msg: String,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut response = ShowWebPage::new();
    response.set_url(msg);
    let res_bytes = response.write_to_bytes()?;
    encode_message(MessageType::SHOW_WEB_PAGE.value() as u32, res_bytes).await
}

pub async fn encode_visibility_change(
    msg: bool,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut response = VisibilityChangeNotification::new();
    response.set_visible(msg);
    let res_bytes = response.write_to_bytes()?;
    encode_message(
        MessageType::VISIBILITY_CHANGE_NOTIFICATION.value() as u32,
        res_bytes,
    )
    .await
}

pub async fn encode_game_invite(
    msg: String,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut response = ShowInvitationDialog::new();
    response.set_connection_string(msg);
    let res_bytes = response.write_to_bytes()?;
    encode_message(
        MessageType::SHOW_INVITATION_DIALOG.value() as u32,
        res_bytes,
    )
    .await
}

pub async fn encode_game_join(
    (inviter, client_id, connection_string): (u64, String, String),
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut response = GameJoinRequestNotification::new();
    response.set_inviter_id(inviter);
    response.set_client_id(client_id);
    response.set_connection_string(connection_string);
    let res_bytes = response.write_to_bytes()?;
    encode_message(
        MessageType::GAME_JOIN_REQUEST_NOTIFICATION.value() as u32,
        res_bytes,
    )
    .await
}

pub async fn encode_overlay_initialized(
    data: Vec<u8>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    encode_message(
        MessageType::OVERLAY_INITIALIZED_NOTIFICATION.value() as u32,
        data,
    )
    .await
}
