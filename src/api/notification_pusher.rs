use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use protobuf::{Enum, Message, UnknownValueRef};
use tokio::net::TcpStream;
use tokio::sync::broadcast::Sender;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_util::sync::CancellationToken;

use crate::proto::common_utils::ProtoPayload;
use crate::proto::gog_protocols_pb::response::Status;
use crate::proto::{
    galaxy_protocols_webbroker_service::{
        AuthRequest, MessageSort, MessageType, SubscribeTopicRequest, SubscribeTopicResponse,
    },
    gog_protocols_pb::Header,
};

pub struct NotificationPusherClient {
    pusher_connection: WebSocketStream<MaybeTlsStream<TcpStream>>,
    access_token: String,
    topic_sender: Sender<Vec<u8>>,
}

impl NotificationPusherClient {
    pub async fn new(
        access_token: &String,
        topic_sender: Sender<Vec<u8>>,
    ) -> NotificationPusherClient {
        let mut retries = 5;
        let ws_stream = loop {
            let stream = NotificationPusherClient::init_connection(access_token).await;
            if let Ok(stream) = stream {
                break stream;
            } else if retries > 0 {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                retries -= 1;
            } else {
                panic!("Failed to create pusher connection");
            }
        };

        NotificationPusherClient {
            pusher_connection: ws_stream,
            access_token: access_token.clone(),
            topic_sender,
        }
    }

    async fn init_connection(
        access_token: &String,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Error> {
        let (mut ws_stream, _) =
            connect_async(crate::constants::NOTIFICATIONS_PUSHER_SOCKET).await?;
        info!("Connected to notifications-pusher");

        let mut header = Header::new();
        header.set_sort(MessageSort::MESSAGE_SORT as u32);
        header.set_type(MessageType::AUTH_REQUEST as u32);

        let mut request_body = AuthRequest::new();
        let token_payload = format!("Bearer {}", access_token);
        request_body.set_auth_token(token_payload);

        let request_body = request_body.write_to_bytes().unwrap();

        header.set_size(request_body.len() as u32);
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

    pub async fn handle_loop(&mut self, shutdown_token: CancellationToken) {
        loop {
            loop {
                let message = tokio::select! {
                    msg = self.pusher_connection.next() => {msg}
                    _ = shutdown_token.cancelled() => {
                        // We are shutting down, we can ignore any errors
                        let _ = self.pusher_connection.close(None).await;
                        None
                    }
                };

                let message = match message {
                    Some(msg) => msg,
                    None => break,
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
                    let msg_type = parsed_message.header.type_();
                    let sort = parsed_message.header.sort();

                    if sort != MessageSort::MESSAGE_SORT as u32 {
                        warn!("Notifications pusher sort has unexpected value {}, ignoring... this may introduce unexpected behavior", sort);
                    }

                    if msg_type == MessageType::AUTH_RESPONSE as u32 {
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
                                    header.set_sort(MessageSort::MESSAGE_SORT as u32);
                                    header.set_type(MessageType::SUBSCRIBE_TOPIC_REQUEST as u32);
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
                    } else if msg_type == MessageType::SUBSCRIBE_TOPIC_RESPONSE as u32 {
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
                    } else if msg_type == MessageType::MESSAGE_FROM_TOPIC as u32 {
                        info!("Recieved message from topic");
                        if let Err(error) = self.topic_sender.send(msg_data) {
                            error!(
                                "There was an error when forwarding topic message: {}",
                                error
                            );
                        }
                    } else {
                        warn!("Unhandled message type: {}", msg_type);
                    }
                }
            }
            if !shutdown_token.is_cancelled() {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                let mut retries = 5;
                let connection = loop {
                    let stream =
                        NotificationPusherClient::init_connection(&self.access_token).await;
                    if let Ok(stream) = stream {
                        break stream;
                    } else if retries > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                        retries -= 1;
                    }
                };
                self.pusher_connection = connection;
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
