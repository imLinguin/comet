use protobuf_codegen::Codegen;

fn main() {
    Codegen::new()
        .protoc()
        .protoc_path(&protoc_bin_vendored::protoc_bin_path().unwrap())
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
