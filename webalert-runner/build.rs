fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .format(true)
        .build_server(false)
        .build_client(true)
        .compile(
            &["../webalert/proto/webalert/runner/v1/runner.proto"],
            &["../webalert/proto"],
        )?;

    Ok(())
}
