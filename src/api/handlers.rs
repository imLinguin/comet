pub mod error;
pub mod utils;
mod webbroker;

use crate::constants::TokenStorage;
use error::*;

use log::{debug, error, info, warn};
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
    mut topic_receiver: Receiver<Vec<u8>>,
    shutdown_token: CancellationToken,
) {
    let mut size_buffer: [u8; 2] = [0; 2];
    if let Err(err) = socket.readable().await {
        error!("Failed to wait for socket to be readable {}", err);
        let _ = socket.shutdown().await;
        return;
    }
    loop {
        tokio::select! {
          size_read = socket.read_exact(&mut size_buffer) => {
            match size_read {
               Ok(_) => {
                   let h_size = u16::from_be_bytes(size_buffer);
                   handle_message(h_size, &mut socket, &reqwest_client).await;
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
                Ok(message) => { if let Err(err) = socket.write_all(message.as_slice()).await {error!("Failed to forward topic message to socket {}", err);} }
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
    socket: &mut TcpStream,
    reqwest_client: &Client,
) -> Result<(), MessageHandlingError> {
    let payload = utils::parse_payload(h_size, socket).await;

    let payload = match payload {
        Ok(p) => p,
        Err(error) => {
            return Err(error::MessageHandlingError::new(
                error::MessageHandlingErrorKind::IO(error),
            ));
        }
    };
    let sort = payload.header.sort();
    let type_ = payload.header.type_();

    debug!("Parsing message {} {}", sort, type_);
    match sort {
        1 => {
            // Communication service
            todo!()
        }
        2 => webbroker::entry_point(payload, reqwest_client).await,
        _ => {
            warn!("Unhandled sort {}", sort);
            Err(MessageHandlingError::new(
                MessageHandlingErrorKind::NotImplemented,
            ))
        }
    };
    Ok(())
}
