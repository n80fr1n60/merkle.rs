#[cfg(feature = "serialization-protobuf")]
fn build_protobuf_schemata() {
    protobuf_codegen::Codegen::new()
        .pure()
        .out_dir("src/proto")
        .input("protobuf/proof.proto")
        .include("protobuf")
        .run()
        .expect("protobuf codegen failed");
}

fn main() {
    #[cfg(feature = "serialization-protobuf")]
    build_protobuf_schemata();
}
