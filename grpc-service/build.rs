use std::{error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from("./src/generated");
    fs::create_dir_all(&out_dir)?;

    tonic_build::configure()
        .protoc_arg("--proto_path=proto")
        .file_descriptor_set_path(out_dir.join("descriptor.bin"))
        .out_dir(&out_dir)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .build_client(false)
        .compile_protos(&["proto/markets.proto"], &["proto"])?;

    println!("cargo:rerun-if-changed=proto/");

    Ok(())
}
