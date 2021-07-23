use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_path =
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("runners_descriptor.bin");

    tonic_build::configure()
        .format(true)
        .file_descriptor_set_path(&descriptor_path)
        .compile(&["proto/webalert/runner/v1/runner.proto"], &["proto"])?;

    Ok(())
}
