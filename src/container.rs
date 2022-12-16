use crate::child::create_child_process;
use crate::cli::BowlArg;
use crate::config_opts::ContainerOptions;
use crate::errors::Errcode;
use crate::mount::clean_mount;

use nix::sys::wait::waitpid;
use nix::unistd::close;
use nix::unistd::Pid;
use std::os::unix::io::RawFd;

use anyhow::{self};
use log::{debug, error};

pub struct BowlContainer {
    sockets: (RawFd, RawFd),
    config: ContainerOptions,
    child_pid: Option<Pid>,
}

impl BowlContainer {
    ///ContainerOptionsのCLI引数から構造体を作成する
    pub fn new(args: BowlArg) -> anyhow::Result<BowlContainer> {
        let (config, sockets) =
            ContainerOptions::new(args.command, args.uid, args.mount_directory)?;
        Ok(BowlContainer {
            sockets,
            config,
            child_pid: None,
        })
    }

    ///createコンテナのプロセスをcreate
    pub fn create_process(&mut self) -> anyhow::Result<()> {
        debug!("create container start");
        let pid = create_child_process(self.config.clone())?;
        self.child_pid = Some(pid);
        debug!("create container finished");
        Ok(())
    }

    ///exit前に呼び出して状態をcleanにする
    pub fn clean(&mut self) -> anyhow::Result<()> {
        debug!("cleanup container");
        if let Err(e) = close(self.sockets.0) {
            error!("Unable to close write socket: {:?}", e);
            return Err(Errcode::SocketError(3).into());
        }

        if let Err(e) = close(self.sockets.1) {
            error!("Unable to close read socket: {:?}", e);
            return Err(Errcode::SocketError(4).into());
        }
        Ok(())
    }

    pub fn clean_exit(&mut self) -> anyhow::Result<()> {
        clean_mount(&self.config.mount_directory)?;

        Ok(())
    }
}

///startから引数を取得してContainer作成から終了まですべてを処理
pub fn start(args: BowlArg) -> anyhow::Result<()> {
    let mut container = BowlContainer::new(args)?;
    debug!(
        "Container sockets: ({}, {})",
        container.sockets.0, container.sockets.1
    );
    if let Err(e1) = container.create_process() {
        error!("Error while create process : {:?}", e1);
        container.clean().map_err(|e2| {
            error!("Error while create container: {:?}", e2);
            Errcode::CleanupFailure(e2)
        })?;
    }
    debug!("Container child PID: {:?}", container.child_pid);
    wait(container.child_pid)?;
    debug!("Success, cleanup and exit");
    container.clean()
}

///child processを作成して終了するまでwait
pub fn wait(pid: Option<Pid>) -> anyhow::Result<()> {
    if let Some(child_pid) = pid {
        debug!("Wait for child (pid {}) to finish", child_pid);
        if let Err(e) = waitpid(child_pid, None) {
            error!("Error while waiting for pid to finish: {:?}", e);
            return Err(Errcode::ContainerError(1).into());
        }
    }
    Ok(())
}
