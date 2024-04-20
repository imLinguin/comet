use protobuf_codegen::Codegen;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=proto/galaxy.protocols.communication_service.proto");
    println!("cargo:rerun-if-env-changed=PROTOC");
    let vendored_protoc = protoc_bin_vendored::protoc_bin_path().unwrap();
    let protoc_path = match std::env::var("PROTOC") {
        Ok(protoc) => PathBuf::from(&protoc),
        Err(_) => vendored_protoc,
    };
    Codegen::new()
        .protoc()
        .protoc_path(protoc_path.as_path())
        .includes(["proto"])
        .input("proto/gog.protocols.pb.proto")
        .input("proto/galaxy.protocols.webbroker_service.proto")
        .input("proto/galaxy.protocols.overlay_for_peer.proto")
        .input("proto/galaxy.protocols.communication_service.proto")
        .input("proto/galaxy.common.protocols.peer_to_server.proto")
        .input("proto/galaxy.common.protocols.peer_to_peer.proto")
        .input("proto/galaxy.common.protocols.peer_common.proto")
        .input("proto/galaxy.common.protocols.connection.proto")
        .cargo_out_dir("proto")
        .run_from_script();
}
