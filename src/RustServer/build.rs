fn main() {
    tonic_build::compile_protos("../Protos/zilean_rust.proto")
        .expect("Failed to compile proto");
}
