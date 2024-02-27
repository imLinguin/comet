use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use protobuf::{Enum, Message, UnknownValueRef};
use tokio::net::TcpStream;
use tokio::sync::broadcast::Sender;
use tokio::time;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_util::sync::CancellationToken;

use crate::proto::common_utils::ProtoPayload;
use crate::proto::gog_protocols_pb::response::Status;
use crate::proto::{
    galaxy_common_protocols_connection,
    galaxy_protocols_webbroker_service::{
        AuthRequest, MessageSort, MessageType, SubscribeTopicRequest, SubscribeTopicResponse,
    },
    gog_protocols_pb::Header,
};

#[derive(Clone)]
pub enum PusherEvent {
    Online,
    Offline,
    Topic(Vec<u8>),
}

pub struct NotificationPusherClient {
    pusher_connection: WebSocketStream<MaybeTlsStream<TcpStream>>,
    access_token: String,
    topic_sender: Sender<PusherEvent>,
    shutdown_token: CancellationToken,
}

impl NotificationPusherClient {
    pub async fn new(
        access_token: &String,
        topic_sender: Sender<PusherEvent>,
        shutdown_token: CancellationToken,
    ) -> NotificationPusherClient {
        debug!("Notification pusher init");
        let mut retries = 5;
        let ws_stream = loop {
            let stream = NotificationPusherClient::init_connection(access_token).await;
            match stream {
                Ok(stream) => break Some(stream),
                Err(tungstenite::Error::Io(_err)) => {
                    tokio::select! {
                        _ = time::sleep(time::Duration::from_secs(10)) => {},
                        _ = shutdown_token.cancelled() => { break None }
                    }
                }
                Err(err) => {
                    if retries > 0 {
                        tokio::select! {
                            _ = time::sleep(time::Duration::from_secs(3)) => {},
                            _ = shutdown_token.cancelled() => { break None }
                        }
                        retries -= 1;
                    } else {
                        panic!("Notification pusher init failed, {:?}", err);
                    }
                }
            }
        };

        NotificationPusherClient {
            pusher_connection: ws_stream.expect("Unable to get notification pusher connection"),
            access_token: access_token.clone(),
            topic_sender,
            shutdown_token,
        }
    }

    async fn init_connection(
        access_token: &String,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Error> {
        let (mut ws_stream, _) =
            connect_async(crate::constants::NOTIFICATIONS_PUSHER_SOCKET).await?;
        info!("Connected to notifications-pusher");

        let mut header = Header::new();
        header.set_sort(MessageSort::MESSAGE_SORT.value().try_into().unwrap());
        header.set_type(MessageType::AUTH_REQUEST.value().try_into().unwrap());

        let mut request_body = AuthRequest::new();
        let token_payload = format!("Bearer {}", access_token);
        request_body.set_auth_token(token_payload);

        let request_body = request_body.write_to_bytes().unwrap();

        header.set_size(request_body.len().try_into().unwrap());
        header.set_oseq(10000);
        let header_data = header.write_to_bytes().unwrap();
        let size: u16 = header_data.len().try_into().unwrap();
        let size_data = size.to_be_bytes();

        let mut buffer = Vec::new();
        buffer.extend(size_data);
        buffer.extend(header_data);
        buffer.extend(request_body);

        let message = tungstenite::Message::Binary(buffer);

        ws_stream.send(message).await?;

        info!("Sent authorization data");
        Ok(ws_stream)
    }

    pub async fn handle_loop(&mut self) {
        loop {
            let mut pending_ping = false;
            loop {
                let message = tokio::select! {
                    msg = self.pusher_connection.next() => {
                        if msg.is_none() {
                            debug!("Connection reset");
                            break;
                        }
                        msg.unwrap()
                    }
                    _ = time::sleep(time::Duration::from_secs(60)) => {
                        if pending_ping {
                            // Send offline status to contexts
                            if let Err(err) = self.topic_sender.send(PusherEvent::Offline) {
                                warn!("Failed to send offline event to contexts {}", err.to_string());
                            }
                            break 
                        }
                        // Ping the service to see if we are still online
                        let mut header = Header::new();
                        header.set_type(galaxy_common_protocols_connection::MessageType::PING.value().try_into().unwrap());
                        header.set_sort(MessageSort::MESSAGE_SORT.value().try_into().unwrap());
                        let mut content = galaxy_common_protocols_connection::Ping::new();
                        content.set_ping_time(chrono::Utc::now().timestamp().try_into().unwrap());
                        let content_buffer = content.write_to_bytes().unwrap();
                        header.set_size(content_buffer.len().try_into().unwrap());
                        let header_buffer = header.write_to_bytes().unwrap();
                        let header_size: u16 = header_buffer.len().try_into().unwrap();

                        let mut message: Vec<u8> = Vec::new();
                        message.extend(header_size.to_be_bytes().to_vec());
                        message.extend(header_buffer);
                        message.extend(content_buffer);

                        let ws_message = tungstenite::Message::Ping(message);
                        if let Err(err) = self.pusher_connection.send(ws_message).await {
                            warn!("Pusher ping failed {:?}", err);
                            break;
                        }
                        pending_ping = true;
                        continue
                    }
                    _ = self.shutdown_token.cancelled() => {
                        // We are shutting down, we can ignore any errors
                        let _ = self.pusher_connection.close(None).await;
                        break;
                    }
                };

                let message = match message {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!(
                            "There was an error reading notifications pusher message: {}",
                            err
                        );
                        continue;
                    }
                };

                debug!("Received a message");
                if message.is_binary() {
                    let msg_data = message.into_data();
                    let proto_message = NotificationPusherClient::parse_message(&msg_data);
                    let parsed_message = match proto_message {
                        Ok(message) => message,
                        Err(err) => {
                            error!("There was an error parsing socket message: {}", err);
                            continue;
                        }
                    };
                    let msg_type: i32 = parsed_message.header.type_().try_into().unwrap();
                    let sort: i32 = parsed_message.header.sort().try_into().unwrap();

                    if sort != MessageSort::MESSAGE_SORT.value() {
                        warn!("Notifications pusher sort has unexpected value {}, ignoring... this may introduce unexpected behavior", sort);
                    }

                    if msg_type == MessageType::AUTH_RESPONSE.value() {
                        // No content
                        let status_code = parsed_message
                            .header
                            .special_fields
                            .unknown_fields()
                            .get(101);
                        if let Some(UnknownValueRef::Varint(code)) = status_code {
                            let code: i32 = code.try_into().unwrap();
                            if let Some(enum_code) = Status::from_i32(code) {
                                if enum_code == Status::OK {
                                    info!("Subscribing to chat, friends, presence");
                                    let mut header = Header::new();
                                    header.set_sort(
                                        MessageSort::MESSAGE_SORT.value().try_into().unwrap(),
                                    );
                                    header.set_type(
                                        MessageType::SUBSCRIBE_TOPIC_REQUEST
                                            .value()
                                            .try_into()
                                            .unwrap(),
                                    );
                                    let mut oseq = 1020;
                                    for topic in ["chat", "friends", "presence"] {
                                        let mut message_buffer: Vec<u8> = Vec::new();
                                        let mut request_data = SubscribeTopicRequest::new();
                                        request_data.set_topic(String::from(topic));
                                        let payload = request_data.write_to_bytes().unwrap();
                                        header.set_size(payload.len().try_into().unwrap());
                                        header.set_oseq(oseq);
                                        oseq += 1;
                                        let header_buf = header.write_to_bytes().unwrap();

                                        let header_size: u16 = header_buf.len().try_into().unwrap();

                                        message_buffer.extend(header_size.to_be_bytes());
                                        message_buffer.extend(header_buf);
                                        message_buffer.extend(payload);

                                        let new_message =
                                            tungstenite::Message::Binary(message_buffer);
                                        if let Err(error) =
                                            self.pusher_connection.feed(new_message).await
                                        {
                                            error!(
                                                "There was an error subscribing to {}, {:?}",
                                                topic, error
                                            );
                                        }
                                    }
                                    if let Err(error) = self.pusher_connection.flush().await {
                                        error!("There was an error flushing {:?}", error);
                                    }
                                    info!("Completed subscribe requests");
                                    continue;
                                }
                            }
                        }
                    } else if msg_type == MessageType::SUBSCRIBE_TOPIC_RESPONSE.value() {
                        let topic_response =
                            SubscribeTopicResponse::parse_from_bytes(&parsed_message.payload);
                        match topic_response {
                            Ok(response) => {
                                let topic = response.topic();
                                info!("Successfully subscribed to topic {}", topic);
                            }
                            Err(err) => {
                                error!("Failed to parse topic response payload {:?}", err)
                            }
                        }
                    } else if msg_type == MessageType::MESSAGE_FROM_TOPIC.value() {
                        info!("Recieved message from topic");
                        if let Err(error) = self.topic_sender.send(PusherEvent::Topic(msg_data)) {
                            error!(
                                "There was an error when forwarding topic message: {}",
                                error
                            );
                        }
                    } else {
                        warn!("Unhandled message type: {}", msg_type);
                    }
                } else if message.is_pong() {
                    debug!("Pong received");
                    pending_ping = false;
                    if let Err(err) = self.topic_sender.send(PusherEvent::Online) {
                        warn!(
                            "Failed to notify handlers about going online {}",
                            err.to_string()
                        );
                    }
                }
            }
            if !self.shutdown_token.is_cancelled() {
                tokio::time::sleep(time::Duration::from_secs(5)).await;
                let mut retries = 5;
                let connection = loop {
                    if self.shutdown_token.is_cancelled() {
                        break None;
                    }
                    let stream =
                        NotificationPusherClient::init_connection(&self.access_token).await;
                    if let Ok(stream) = stream {
                        break Some(stream);
                    } else if retries > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                        retries -= 1;
                    }
                };
                if let Some(connection) = connection {
                    self.pusher_connection = connection;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub fn parse_message(msg_data: &Vec<u8>) -> Result<ProtoPayload, protobuf::Error> {
        let data = msg_data.as_slice();

        let mut header_size_buf = [0; 2];
        header_size_buf.copy_from_slice(&data[..2]);
        let header_size = u16::from_be_bytes(header_size_buf).into();

        let mut header_buf: Vec<u8> = vec![0; header_size];
        header_buf.copy_from_slice(&data[2..header_size + 2]);

        let header = Header::parse_from_bytes(&header_buf)?;

        let payload_size = header.size().try_into().unwrap();
        let mut payload: Vec<u8> = vec![0; payload_size];

        payload.copy_from_slice(&data[header_size + 2..header_size + 2 + payload_size]);

        Ok(ProtoPayload { header, payload })
    }
}
