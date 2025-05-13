use std::{fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from("./src/generated");
    fs::create_dir_all(&out_dir)?;

    let mut config = prost_build::Config::new();

    config
        .protoc_arg("--proto_path=proto")
        .out_dir(&out_dir)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(&["proto/markets.proto"], &["proto"])?;

    println!("cargo:rerun-if-changed=proto/");

    Ok(())
}
