use std::path::PathBuf;

fn main() {
    let proto_dir = PathBuf::from("../../proto");
    let agents_proto = proto_dir.join("agents.proto");

    prost_build::Config::new()
        .compile_protos(&[agents_proto], &[proto_dir])
        .expect("Failed to compile protos");
}