mod cli;
mod database;
mod migration;

use log::{debug, error};
use structopt::StructOpt;
use tokio::runtime::Runtime;

async fn async_main(opts: cli::Opts) -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the PostgreSQL database
    let mut conn = database::init(&opts).await?;

    match &opts.command {
        cli::Command::Server(_) => {
            debug!("Starting server");
        }
        cli::Command::DbCommand(cmd) => match &cmd {
            cli::DbSubCommand::Migrate(dir) => {
                database::init_migration(&mut conn).await?;

                match dir {
                    cli::MigrateCommand::Up(_) => {
                        debug!("Migration files: {:?}", migration::get_migrations_sorted()?);
                        debug!("db_conn: {:?}", conn);
                    }
                    cli::MigrateCommand::Down(_) => {}
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
        _ => unimplemented!(),
    }
}
