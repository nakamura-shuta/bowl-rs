mod cli;
mod ipc;
mod config_opts;
mod container;
mod errors;

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
