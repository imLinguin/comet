use log::info;
use protobuf::Message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::proto::{
    galaxy_protocols_webbroker_service::{AuthRequest, MessageSort, MessageType},
    gog_protocols_pb::Header,
};

pub struct NotificationPusherClient {
    pusher_connection: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl NotificationPusherClient {
    pub async fn new(access_token: &String) -> NotificationPusherClient {
        let (mut ws_stream, _) = connect_async("wss://notifications-pusher.gog.com")
            .await
            .expect("Failed to connect");
        info!("Connected to notifications-pusher");

        let stream = ws_stream.get_mut();

        let mut header = Header::new();
        header.set_sort(MessageSort::MESSAGE_SORT as u32);
        header.set_type(MessageType::AUTH_REQUEST as u32);

        let mut request_body = AuthRequest::new();
        request_body.set_auth_token(format!("Bearer {access_token}"));

        let request_body = request_body.write_to_bytes().unwrap();

        header.set_size(request_body.len() as u32);
        let header_data = header.write_to_bytes().unwrap();
        let size = header_data.len().to_be_bytes();

        let mut buffer = Vec::new();
        buffer.extend_from_slice(&size);
        buffer.extend_from_slice(&header_data);
        buffer.extend_from_slice(&request_body);

        stream
            .write_all(&buffer)
            .await
            .expect("Failed to write into socket");
        info!("Sent authorization data");

        NotificationPusherClient {
            pusher_connection: ws_stream,
        }
    }

    pub async fn handle_loop(&mut self) {}
}
