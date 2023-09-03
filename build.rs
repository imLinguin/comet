use protobuf_codegen::Codegen;

fn main() {
    Codegen::new()
        .protoc()
        .protoc_path(&protoc_bin_vendored::protoc_bin_path().unwrap())
        .includes(&["src/proto"])
        .input("src/proto/gog.protocols.pb.proto")
        .input("src/proto/galaxy.protocols.webbroker_service.proto")
        .input("src/proto/galaxy.protocols.overlay_for_peer.proto")
        .input("src/proto/galaxy.protocols.communication_service.proto")
        .input("src/proto/galaxy.common.protocols.peer_to_server.proto")
        .input("src/proto/galaxy.common.protocols.peer_to_peer.proto")
        .input("src/proto/galaxy.common.protocols.peer_common.proto")
        .input("src/proto/galaxy.common.protocols.connection.proto")
        .cargo_out_dir("proto")
        .run_from_script();
}
