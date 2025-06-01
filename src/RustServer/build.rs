use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let descriptor_path = out_dir.join("descriptor.bin");

    let mut config = prost_build::Config::new();
    config.file_descriptor_set_path(&descriptor_path);

    // Enable gRPC codegen
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos_with_config(
            config,
            &["../Protos/zilean_rust.proto"],
            &["../Protos"],
        )?;

    Ok(())
}