fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic-build generates Rust code from proto/asi.proto.
    // Uncomment when tonic/prost are fully integrated:
    // tonic_build::compile_protos("../../proto/asi.proto")?;
    Ok(())
}
