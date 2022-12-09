mod cli;
mod config_opts;
mod child;
mod container;
mod errors;
mod ipc;

use cli::parse_args;
use log::{error, info};

fn main() -> anyhow::Result<()> {
    match parse_args() {
        Ok(args) => {
            info!("cli args : {:?}", args);
            //exit_with_retcode(Ok(()))
            container::start(args);
            Ok(())
        }
        Err(err) => {
            error!("Error occurred while parsing arguments -> {}", err);
            anyhow::bail!("Process Execute Error")
        }
    }
}
