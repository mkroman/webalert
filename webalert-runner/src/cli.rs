use structopt::StructOpt;
use url::Url;

#[derive(Debug, StructOpt)]
pub struct Opts {
    /// gRPC host to connect to
    #[structopt(
        long = "grpc-server-url",
        env = "WEBALERT_GRPC_URL",
        default_value = "http://[::]:3031"
    )]
    pub grpc_url: String,

    /// The address of the webdriver to use
    #[structopt(
        long = "webdriver-url",
        env = "WEBALERT_WEBDRIVER_URL",
        required = true
    )]
    pub webdriver_url: Url,

    /// The runners authorization token
    #[structopt(long = "grpc-token", env = "WEBALERT_GRPC_TOKEN", required = true)]
    pub grpc_token: String,
}
