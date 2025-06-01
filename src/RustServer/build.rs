fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();
    config.file_descriptor_set_path("descriptor.bin");

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