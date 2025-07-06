fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/wagering.proto")?;
    tonic_build::compile_protos("proto/admin.proto")?;
    tonic_build::compile_protos("proto/draw.proto")?;
    Ok(())
}
