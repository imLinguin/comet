use protobuf::Message;
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::proto::{common_utils::ProtoPayload, gog_protocols_pb};

pub async fn parse_payload(
    h_size: u16,
    socket: &mut TcpStream,
) -> Result<ProtoPayload, tokio::io::Error> {
    let mut buffer: Vec<u8> = Vec::new();
    buffer.reserve(h_size.into());

    socket.read_exact(&mut buffer).await?;

    let header = gog_protocols_pb::Header::parse_from_bytes(&buffer)?;

    let size = header.size();
    buffer.reserve(size.try_into().unwrap());
    socket.read_exact(&mut buffer).await?;
    Ok(ProtoPayload {
        header,
        payload: buffer,
    })
}
