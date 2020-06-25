use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opts {
    /// The number of webdrivers to run in parallel
    #[structopt(short = "n", default_value = "3")]
    pub num_webdrivers: u64,
}
