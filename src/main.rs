use webalert::http;
use webalert::migration::MigrationRunner;
use webalert::{cli, database};

use log::{debug, error};
use structopt::StructOpt;
use tokio::runtime::Runtime;

async fn async_main(opts: cli::Opts) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the PostgreSQL database
    let mut pool = database::init(&opts).await?;

    match &opts.command {
        cli::Command::Server(ref server_opts) => {
            debug!("Starting server");

            let http_server = http::start_http_server(server_opts, pool);

            tokio::join!(http_server);
        }
        cli::Command::DbCommand(cmd) => match &cmd {
            cli::DbSubCommand::Migrate(dir) => {
                // Create the necessary database schema for migrations if it doesn't exist
                database::init_migration(&mut pool).await?;

                let current_version = database::get_migration_version(&pool).await?;
                let mut runner = MigrationRunner::new(&mut pool, current_version);

                match dir {
                    cli::MigrateCommand::Up(ver) => {
                        runner.migrate_up_to_version(ver.version.as_deref()).await?;
                    }
                    cli::MigrateCommand::Down(ver) => {
                        runner.migrate_down_to_version(ver.version.as_ref()).await?;
                    }
                }
            }
        },
    }

    Ok(())
}

fn main() {
    env_logger::init();

    // Set up the async runtime
    let mut rt = Runtime::new().expect("unable to create runtime");
    // Parse the command-line arguments
    let opts = cli::Opts::from_args();

    match &opts.command {
        cli::Command::Server(_) | cli::Command::DbCommand(_) => {
            if let Err(err) = rt.block_on(async_main(opts)) {
                error!("runtime error: {}", err);
            }
        }
    }
}
