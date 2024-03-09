mod communication_service;
pub mod context;
pub mod error;
pub mod utils;
mod webbroker;

use crate::constants::TokenStorage;
use error::*;
use std::sync::Arc;

use crate::api::gog;
use crate::api::notification_pusher::PusherEvent;
use crate::api::structs::UserInfo;
use crate::db;
use context::HandlerContext;
use log::{debug, error, info, warn};
use protobuf::Message;
use reqwest::Client;
use sqlx::Acquire;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::broadcast::Receiver,
    time,
};
use tokio_util::sync::CancellationToken;

pub async fn entry_point(
    mut socket: TcpStream,
    reqwest_client: Client,
    token_store: TokenStorage,
    user_info: Arc<UserInfo>,
    mut topic_receiver: Receiver<PusherEvent>,
    shutdown_token: CancellationToken,
) {
    if let Err(err) = socket.readable().await {
        error!("Failed to wait for socket to be readable {}", err);
        let _ = socket.shutdown().await;
        return;
    }
    let mut context = HandlerContext::new(socket, token_store);
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

            _ = time::sleep(time::Duration::from_secs(10)) => {
                sync_routine(&mut context, &reqwest_client, user_info.clone()).await
            },

            topic_message = topic_receiver.recv() => {
                match topic_message {
                    Ok(PusherEvent::Online) => {
                            context.set_online()
                    },
                    Ok(PusherEvent::Offline) => {
                        context.set_offline()
                    },
                    Ok(PusherEvent::Topic(message)) => {
                        if let Err(err) = context.socket_mut().write_all(message.as_slice()).await {
                            error!("Failed to forward topic message to socket {}", err);
                        }
                    },
                    Err(err) => { error!("Failed to read topic_message {}", err); }
                }
            }

            _ = shutdown_token.cancelled() => {
                break
            }
        }
    }
    sync_routine(&mut context, &reqwest_client, user_info.clone()).await;
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
    result.header.set_sort(sort);
    debug!(
        "Responding with {} {}",
        result.header.sort(),
        result.header.type_()
    );
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
        Vec::with_capacity(2 + header_buffer.len() + result.payload.len());
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

// Sync new things after a cool down or when about to exit
async fn sync_routine(
    context: &mut HandlerContext,
    reqwest_client: &Client,
    user_info: Arc<UserInfo>,
) {
    // Make sure we are online
    if !context.is_online() {
        return;
    }
    let mut token_store = context.token_store().lock().await;
    let client_id = &context.client_id().clone().unwrap();
    let client_secret = &context.client_secret().clone().unwrap();
    let current_token = token_store.get(client_id);
    if let Some(token) = current_token {
        let current_time = chrono::Utc::now();
        if (current_time.timestamp() - token.obtain_time.timestamp()) >= 3500 {
            debug!("Refreshing credentials for {}", client_id);
            let result = gog::users::get_token_for(
                client_id,
                client_secret,
                &token.refresh_token,
                reqwest_client,
            )
            .await;
            match result {
                Ok(new_token) => {
                    token_store.insert(client_id.clone(), new_token);
                }
                Err(err) => {
                    drop(token_store);
                    if err.is_connect() || err.is_timeout() {
                        context.set_offline();
                    }
                    warn!("Failed to refresh the token for {} {:?}", client_id, err);
                    return;
                }
            }
        }
    }
    drop(token_store);
    let updated_achievements = *context.updated_achievements();
    let updated_stats = *context.updated_stats();
    // Is there anything to update?
    if !(updated_achievements || updated_stats) {
        return;
    }

    if *context.updated_achievements() {
        // Sync achievements
        info!("Uploading new achievements");
        let changed_achievements = db::gameplay::get_achievements(context, true).await;
        match changed_achievements {
            Ok((achievements, _mode)) => {
                let db = context.db_connection();
                let mut connection = db
                    .acquire()
                    .await
                    .expect("Failed to get database connection");
                let mut transaction = connection.begin().await.unwrap();

                for achievement in achievements {
                    debug!("Setting achievement {}", achievement.achievement_key());
                    let result = gog::achievements::set_achievement(
                        context,
                        reqwest_client,
                        &user_info.galaxy_user_id,
                        achievement.achievement_id(),
                        achievement.date_unlocked().to_owned(),
                    )
                    .await;
                    if result.is_ok() {
                        // Update local entry with changed to false
                        let a_id: i64 = achievement.achievement_id().parse().unwrap();
                        sqlx::query("UPDATE achievement SET changed=0 WHERE id=$1")
                            .bind(a_id)
                            .execute(&mut *transaction)
                            .await
                            .expect("Failed to update changed status");
                    }
                }
                transaction.commit().await.expect("Failed to save changes");
                context.set_updated_achievements(false);
            }
            Err(err) => error!("Failed to read local database {:?}", err),
        }
        info!("Uploaded");
    }

    if *context.updated_stats() {
        // Sync stats
        info!("Uploading new stats");
        let changed_statistics = db::gameplay::get_statistics(context, true).await;
        match changed_statistics {
            Ok(stats) => {
                let db = context.db_connection();
                let mut connection = db
                    .acquire()
                    .await
                    .expect("Failed to get database connection");
                let mut transaction = connection
                    .begin()
                    .await
                    .expect("Failed to start transaction");

                for stat in stats {
                    debug!("Setting stat {}", stat.stat_id());
                    let result = gog::stats::update_stat(
                        context,
                        reqwest_client,
                        &user_info.galaxy_user_id,
                        &stat,
                    )
                    .await;

                    if result.is_ok() {
                        // Update local entry with changed to false
                        let a_id: i64 = stat.stat_id().parse().unwrap();
                        sqlx::query("UPDATE statistic SET changed=0 WHERE id=$1")
                            .bind(a_id)
                            .execute(&mut *transaction)
                            .await
                            .expect("Failed to update changed status");
                    }
                }
                transaction.commit().await.expect("Failed to save changes");
                context.set_updated_stats(false);
            }
            Err(err) => error!("Failed to read local database {:?}", err),
        }
    }
}
