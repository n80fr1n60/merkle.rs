#[cfg(feature = "serialization-protobuf")]
use std::env;
#[cfg(feature = "serialization-protobuf")]
use std::fs;
#[cfg(feature = "serialization-protobuf")]
use std::path::Path;

#[cfg(feature = "serialization-protobuf")]
const PROTO_FILES: &[&str] = &["protobuf/proof.proto"];

#[cfg(feature = "serialization-protobuf")]
fn path_str(path: &Path) -> &str {
    path.to_str()
        .expect("Generated protobuf output path must be valid UTF-8")
}

#[cfg(feature = "serialization-protobuf")]
fn rust_path_literal(path: &Path) -> String {
    format!("{:?}", path_str(path))
}

#[cfg(feature = "serialization-protobuf")]
fn build_protobuf_schemata(out_dir: &Path) {
    let proto_dir = out_dir.join("proto");
    fs::create_dir_all(&proto_dir).expect("Could not create protobuf output directory");

    protobuf_codegen::Codegen::new()
        .pure()
        .out_dir(path_str(&proto_dir))
        .input("protobuf/proof.proto")
        .include("protobuf")
        .run()
        .expect("protobuf codegen failed");
}

#[cfg(feature = "serialization-protobuf")]
fn write_protobuf_module_descriptor(out_dir: &Path) {
    let descriptor = format!(
        "#[path = {proof_rs}]\npub mod proof;\n",
        proof_rs = rust_path_literal(&out_dir.join("proto").join("proof.rs")),
    );

    fs::write(out_dir.join("proto_mod.rs"), descriptor)
        .expect("Could not write protobuf module descriptor");
}

fn main() {
    #[cfg(feature = "serialization-protobuf")]
    {
        println!("cargo:rerun-if-changed=build.rs");
        for proto in PROTO_FILES {
            println!("cargo:rerun-if-changed={proto}");
        }

        let out_dir = env::var("OUT_DIR").expect("OUT_DIR is not set");
        let out_dir = Path::new(&out_dir);

        build_protobuf_schemata(out_dir);
        write_protobuf_module_descriptor(out_dir);
    }
}
