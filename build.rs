fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["proto/runners.proto"], &["proto"])?;

    Ok(())
}
