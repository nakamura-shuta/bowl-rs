use nix::unistd::close;
use std::os::unix::io::RawFd;

use crate::cli::BowlArg;
use crate::config_opts::ContainerOptions;
use crate::errors::Errcode;

use anyhow::{self};
use log::{debug, error};

pub struct BowlContainer {
    sockets: (RawFd, RawFd),
    config: ContainerOptions,
}

impl BowlContainer {
    ///ContainerOptionsのCLI引数から構造体を作成する
    pub fn new(args: BowlArg) -> anyhow::Result<BowlContainer> {
        let (config,sockets) = ContainerOptions::new(args.command, args.uid, args.mount_directory)?;
        Ok(BowlContainer {
            sockets,
            config,
        })
    }

    ///createコンテナのプロセスをcreate
    pub fn create_process(&mut self) -> anyhow::Result<()> {
        debug!("create container process");
        Ok(())
    }

    ///exit前に呼び出して状態をcleanにする
    pub fn clean(&mut self) -> anyhow::Result<()> {
        debug!("cleanup container");
        Ok(())
    }
}

///startから引数を取得してContainer作成から終了まですべてを処理
pub fn start(args: BowlArg) -> anyhow::Result<()> {
    let mut container = BowlContainer::new(args)?;
    if let Err(e1) = container.create_process() {
        error!("Error while create process : {:?}", e1);
        container.clean().map_err(|e2| {
            error!("Error while create container: {:?}", e2);
            return Errcode::CleanupFailure(e2);
        })?;
    }
    debug!("Success, cleanup and exit");
    container.clean()
}
