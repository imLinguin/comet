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
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::broadcast::Receiver, time};
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
    let context = Arc::new(HandlerContext::new(socket, token_store));
    debug!("Awaiting messages");

    let shutdown_token_clone = shutdown_token.clone();
    let context_clone = context.clone();
    let reqwest_clone = reqwest_client.clone();
    let user_clone = user_info.clone();
    let main_socket = tokio::spawn(async move {
        loop {
            tokio::select! {
                size_read = context_clone.socket_read_u16() => {
                    match size_read {
                        Ok(h_size) => {
                            if let Err(err) = handle_message(h_size, &context_clone, user_clone.clone(), &reqwest_clone).await {
                                match err.kind {
                                    MessageHandlingErrorKind::NotImplemented => {
                                        warn!("Request type not implemented")
                                    },
                                    MessageHandlingErrorKind::Unauthorized => {
                                        let _ = context_clone.socket_mut().await.shutdown().await;
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
                                info!("Socket connection closed with {:?}", context_clone.client_id().await);
                                break;
                            }
                            error!("Was unable to read header size buffer {}", err);
                            break;
                        }
                    }
                }

                topic_message = topic_receiver.recv() => {
                    match topic_message {
                        Ok(PusherEvent::Online) => {
                                context_clone.set_online().await
                        },
                        Ok(PusherEvent::Offline) => {
                            context_clone.set_offline().await
                        },
                        Ok(PusherEvent::Topic(message)) => {
                            if let Err(err) = context_clone.socket_mut().await.write_all(message.as_slice()).await {
                                error!("Failed to forward topic message to socket {}", err);
                            }
                            debug!("Forwarded topic message");
                        },
                        Err(err) => { error!("Failed to read topic_message {}", err); }
                    }
                }

                _ = shutdown_token_clone.cancelled() => break
            }
        }
    });

    let shutdown_token_clone = shutdown_token.clone();
    let context_clone = context.clone();
    let reqwest_clone = reqwest_client.clone();
    let user_clone = user_info.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = time::sleep(time::Duration::from_secs(10)) => {
                    sync_routine(&context_clone, &reqwest_clone, user_clone.clone()).await
                },

                _ = shutdown_token_clone.cancelled() => {
                    break
                }
            }
        }
    });
    let _ = main_socket.await;
    sync_routine(&context, &reqwest_client, user_info.clone()).await;
}

pub async fn handle_message(
    h_size: u16,
    context: &HandlerContext,
    user_info: Arc<UserInfo>,
    reqwest_client: &Client,
) -> Result<(), MessageHandlingError> {
    let payload = utils::parse_payload(h_size, &mut *context.socket_mut().await).await;

    let payload =
        payload.map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::IO(err)))?;

    let sort = payload.header.sort();
    let type_ = payload.header.type_();

    debug!("Parsing message {} {}", sort, type_);
    debug!("payload.payload = {:?}", payload.payload);
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
        .await
        .write_all(message_buffer.as_slice())
        .await
        .map_err(|err| MessageHandlingError::new(MessageHandlingErrorKind::IO(err)))?;
    Ok(())
}

// Sync new things after a cool down or when about to exit
async fn sync_routine(context: &HandlerContext, reqwest_client: &Client, user_info: Arc<UserInfo>) {
    // Make sure we are online
    if !context.is_online().await {
        return;
    }
    let mut token_store = context.token_store().lock().await;
    let client_id = &context.client_id().await.unwrap();
    let client_secret = &context.client_secret().await.unwrap();
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
                token.scope.is_some(),
            )
            .await;
            match result {
                Ok(new_token) => {
                    token_store.insert(client_id.clone(), new_token);
                }
                Err(err) => {
                    drop(token_store);
                    if err.is_connect() || err.is_timeout() {
                        context.set_offline().await;
                    }
                    warn!("Failed to refresh the token for {} {:?}", client_id, err);
                    return;
                }
            }
        }
    }
    drop(token_store);
    let updated_achievements = context.updated_achievements().await;
    let updated_stats = context.updated_stats().await;
    let updated_leaderboards = context.updated_leaderboards().await;
    // Is there anything to update?
    if !(updated_achievements || updated_stats || updated_leaderboards) {
        return;
    }

    if updated_achievements {
        // Sync achievements
        info!("Uploading new achievements");
        let changed_achievements = db::gameplay::get_achievements(context, true).await;
        match changed_achievements {
            Ok((achievements, _mode)) => {
                let db = context.db_connection().await;
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
                context.set_updated_achievements(false).await;
                info!("Uploaded achievements");
            }
            Err(err) => error!("Failed to read local database {:?}", err),
        }
    }

    if updated_stats {
        // Sync stats
        info!("Uploading new stats");
        let changed_statistics = db::gameplay::get_statistics(context, true).await;
        match changed_statistics {
            Ok(stats) => {
                let db = context.db_connection().await;
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
                context.set_updated_stats(false).await;
                info!("Uploaded stats");
            }
            Err(err) => error!("Failed to read local database {:?}", err),
        }
    }

    if updated_leaderboards {
        info!("Syncing leaderboards");

        let changed_leaderboards = db::gameplay::get_leaderboards_score_changed(context).await;

        match changed_leaderboards {
            Ok(entries) => {
                let mut connection = context
                    .db_connection()
                    .await
                    .acquire()
                    .await
                    .expect("Failed to get database connection");
                let mut transaction = connection
                    .begin()
                    .await
                    .expect("Failed to start transaction");

                for (id, score, _rank, _entry_total_count, force, details) in entries {
                    let details = if details.is_empty() {
                        None
                    } else {
                        Some(details)
                    };
                    let result = gog::leaderboards::post_leaderboard_score(
                        context,
                        reqwest_client,
                        &user_info.galaxy_user_id,
                        id,
                        score,
                        force,
                        details,
                    )
                    .await;

                    match result {
                        Ok(res) => {
                            sqlx::query("UPDATE leaderboard SET changed=0,entry_total_count=$2,rank=$3 WHERE id=$1")
                            .bind(id)
                            .bind(res.leaderboard_entry_total_count)
                            .bind(res.new_rank)
                            .execute(&mut *transaction)
                            .await
                            .expect("Failed to update leaderboard state");
                        }
                        Err(err) => {
                            if let Some(status) = err.status() {
                                if status.as_u16() == 409 {
                                    warn!("Leaderboard conflict for {}", id);
                                    let entries = gog::leaderboards::get_leaderboards_entries(
                                        context,
                                        reqwest_client,
                                        id as u64,
                                        [("users", &user_info.galaxy_user_id)],
                                    )
                                    .await;
                                    match entries {
                                        Ok(entries) => {
                                            if let Some(entry) = entries.items.first() {
                                                sqlx::query("UPDATE leaderboard SET changed=0, score=$2, rank=$3 WHERE id=$1")
                                    .bind(id)
                                    .bind(entry.score)
                                    .bind(entry.rank)
                                    .execute(&mut *transaction)
                                    .await
                                    .expect("Failed to set new score locally");
                                            }
                                        }
                                        Err(err) => {
                                            error!("{}", err);
                                            sqlx::query(
                                                "UPDATE leaderboard SET changed=0 WHERE id=$1",
                                            )
                                            .bind(id)
                                            .execute(&mut *transaction)
                                            .await
                                            .expect("Failed to set new score locally");
                                        }
                                    }
                                }
                            }
                            warn!("More details {}", err);
                        }
                    }
                }

                transaction.commit().await.expect("Failed to save changes");
                info!("Leaderboards synced");
                context.set_updated_leaderboards(false).await;
            }
            Err(err) => error!("Failed to read local database {:?}", err),
        }
    }
}
