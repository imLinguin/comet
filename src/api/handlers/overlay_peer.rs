use crate::proto::common_utils::ProtoPayload;
use crate::proto::gog_protocols_pb::Header;
use crate::{api::gog::overlay::OverlayPeerMessage, proto::galaxy_protocols_overlay_for_peer::*};
use log::warn;
use protobuf::{Enum, Message};

use super::{context::HandlerContext, MessageHandlingError, MessageHandlingErrorKind};

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
    } else {
        warn!("Received unsupported peer message type {}", message_type);
        return Err(MessageHandlingError::new(
            MessageHandlingErrorKind::NotImplemented,
        ));
    }
    Err(MessageHandlingError::new(MessageHandlingErrorKind::Ignored))
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
