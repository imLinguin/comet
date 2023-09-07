// Load protobuf
include!(concat!(env!("OUT_DIR"), "/proto/mod.rs"));

pub mod common_utils {
    use protobuf::Message;

    use super::gog_protocols_pb;

    pub struct ProtoPayload {
        pub header: gog_protocols_pb::Header,
        pub payload: Vec<u8>,
    }

    pub async fn parse_message(msg_data: &Vec<u8>) -> Result<ProtoPayload, protobuf::Error> {
        let data = msg_data.as_slice();

        let mut header_size_buf = [0; 2];
        header_size_buf.copy_from_slice(&data[..2]);
        let header_size = u16::from_be_bytes(header_size_buf).into();

        let mut header_buf: Vec<u8> = vec![0; header_size];
        header_buf.copy_from_slice(&data[2..header_size + 2]);

        let header = gog_protocols_pb::Header::parse_from_bytes(&header_buf)?;

        let payload_size = header.size().try_into().unwrap();
        let mut payload: Vec<u8> = vec![0; payload_size];

        payload.copy_from_slice(&data[header_size + 2..header_size + 2 + payload_size]);

        Ok(ProtoPayload { header, payload })
    }
}
