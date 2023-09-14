use log::debug;
use protobuf::Message;
use tokio::{io::AsyncReadExt, net::TcpStream};

use crate::proto::{common_utils::ProtoPayload, gog_protocols_pb};

pub async fn parse_payload(
    h_size: u16,
    socket: &mut TcpStream,
) -> Result<ProtoPayload, tokio::io::Error> {
    let h_size: usize = h_size.into();
    let mut buffer: Vec<u8> = vec![0; h_size];

    socket.read_exact(&mut buffer).await?;

    let header = gog_protocols_pb::Header::parse_from_bytes(&buffer)?;

    let size: usize = header.size().try_into().unwrap();
    buffer.resize(size, 0);
    debug!(
        "Reading payload size {} to buffer of size {}",
        size,
        buffer.len()
    );
    socket.read_exact(&mut buffer).await?;
    Ok(ProtoPayload {
        header,
        payload: buffer,
    })
}
