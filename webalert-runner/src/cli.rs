use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    /// gRPC host to connect to
    #[structopt(
        long = "grpc-server-url",
        env = "WEBALERT_GRPC_URL",
        default_value = "http://[::]:3031"
    )]
    pub grpc_url: String,

    /// The runners authorization token
    #[structopt(long = "grpc-token", env = "WEBALERT_GRPC_TOKEN", required = true)]
    pub grpc_token: String,
}
