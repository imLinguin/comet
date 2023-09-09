// Load protobuf
include!(concat!(env!("OUT_DIR"), "/proto/mod.rs"));

pub mod common_utils {
    pub struct ProtoPayload {
        pub header: super::gog_protocols_pb::Header,
        pub payload: Vec<u8>,
    }
}
