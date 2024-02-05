mod communication_service;
pub mod context;
pub mod error;
pub mod utils;
mod webbroker;

use crate::constants::TokenStorage;
use error::*;
use std::sync::Arc;

use context::HandlerContext;
use crate::api::structs::UserInfo;
use log::{debug, error, info, warn};
use protobuf::Message;
use reqwest::Client;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::broadcast::Receiver,
};
use tokio_util::sync::CancellationToken;

pub async fn entry_point(
    mut socket: TcpStream,
    reqwest_client: Client,
    token_store: TokenStorage,
    user_info: Arc<UserInfo>,
    mut topic_receiver: Receiver<Vec<u8>>,
    shutdown_token: CancellationToken,
) {
    if let Err(err) = socket.readable().await {
        error!("Failed to wait for socket to be readable {}", err);
        let _ = socket.shutdown().await;
        return;
    }
    let mut context = HandlerContext::new(socket, shutdown_token.clone(), token_store);
    debug!("Awaiting messages");
    loop {
        tokio::select! {
          size_read = context.socket_mut().read_u16() => {
            match size_read {
               Ok(h_size) => {
                   if let Err(err) = handle_message(h_size, &mut context, user_info.clone(), &reqwest_client).await {
                            match err.kind {
                                MessageHandlingErrorKind::NotImplemented => {
                                    warn!("Request type not implemented")
                                },
                                MessageHandlingErrorKind::Unauthorized => {
                                    let _ = context.socket_mut().shutdown().await;
                                    return
                                }
                                _ => {
                                    error!("There was an error when handling the message {:?}", err);
                                }
                            };
                        }
               },
               Err(err) => {
                   if err.kind() == tokio::io::ErrorKind::UnexpectedEof {
                    info!("Socket connection closed");
                    break;
                   }
                   error!("Was unable to read header size buffer {}", err);
                   break;
               }
            }
          }

          topic_message = topic_receiver.recv() => {
            match topic_message {
                Ok(message) => { if let Err(err) = context.socket_mut().write_all(message.as_slice()).await {error!("Failed to forward topic message to socket {}", err);} }
                Err(err) => { error!("Failed to read topic_message {}", err); }
            }
          }

          _ = shutdown_token.cancelled() => {
            break
          }
        }
    }
}

pub async fn handle_message(
    h_size: u16,
    context: &mut HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<(), MessageHandlingError> {
    let payload = utils::parse_payload(h_size, context.socket_mut()).await;

    let payload =
        payload.map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::IO(err)))?;

    let sort = payload.header.sort();
    let type_ = payload.header.type_();

    debug!("Parsing message {} {}", sort, type_);
    let mut result = match sort {
        1 => communication_service::entry_point(&payload, context, user_info, reqwest_client).await,
        2 => webbroker::entry_point(&payload).await,
        _ => {
            warn!("Unhandled sort {}", sort);
            Err(MessageHandlingError::new(
                MessageHandlingErrorKind::NotImplemented,
            ))
        }
    }?;
    // Prepare response
    if payload.header.has_oseq() {
        result
            .header
            .mut_unknown_fields()
            .add_varint(100, payload.header.oseq().into());
    }
    let header_buffer = result.header.write_to_bytes().unwrap();
    let header_size: u16 = header_buffer.len().try_into().unwrap();
    let header_buf = header_size.to_be_bytes();

    let mut message_buffer: Vec<u8> =
        Vec::with_capacity(2 + header_buf.len() + payload.payload.len());
    message_buffer.extend(header_buf);
    message_buffer.extend(header_buffer);
    message_buffer.extend(result.payload);

    context
        .socket_mut()
        .write_all(message_buffer.as_slice())
        .await
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::IO(err)))?;
    Ok(())
}
