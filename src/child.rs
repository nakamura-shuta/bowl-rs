use crate::config_opts::ContainerOptions;
use crate::errors::Errcode;
use crate::host::set_container_hostname;
use crate::mount::set_mount_point;
use crate::namespace::user_namespace;
use crate::capa::set_capa;

use nix::unistd::{Pid, close};
use nix::sched::clone;
use nix::sched::CloneFlags;
use nix::sys::signal::Signal;

use log::{error, info};

use anyhow::{self};

const STACK_SIZE: usize = 1024 * 1024;

///initialize Container
fn init_container_config(config: &ContainerOptions) -> anyhow::Result<()> {
    set_container_hostname(&config.hostname)?;
    set_mount_point(&config.mount_directory)?;
    user_namespace(config.fd, config.uid)?;
    set_capa()?;
    Ok(())
}

fn child(config: ContainerOptions) -> isize {
    match init_container_config(&config) {
        Ok(_) => info!("Container init success!"),
        Err(e) => {
            error!("Error while init container: {:?}", e);
            return -1;
        }
    }

    //使用されなくなったらsocket close.
    if let Err(_) = close(config.fd){
        error!("Error while closing socket..");
        return -1;
    }

    info!(
        "Starting container with command {} and args {:?}",
        config.path.to_str().unwrap(),
        config.args
    );
    0
}

///Duplicate the parent process and call the child process
pub fn create_child_process(config: ContainerOptions) -> anyhow::Result<Pid> {
    //child processのスタックcloneを保持するため
    //1kbのraw bufferを割り当て
    let mut tmp_stack: [u8; STACK_SIZE] = [0; STACK_SIZE];

    //Flags definition
    //各フラグを設定してnamespceに対して、child processの新しいnamespaceを作成
    //see:https://docs.rs/nix/0.23.0/nix/sched/struct.CloneFlags.html
    //see:https://man7.org/linux/man-pages/man2/clone.2.html
    let mut flags = CloneFlags::empty();

    //新しいmount namespaceで複製されたchild processを開始
    flags.insert(CloneFlags::CLONE_NEWNS);
    //新しいcgroup namespaceでcloneされたchild processを開始
    flags.insert(CloneFlags::CLONE_NEWCGROUP);
    //新しいpid namespaceでcloneされたchild processを開始
    flags.insert(CloneFlags::CLONE_NEWPID);
    //新しいipc namespaceでcloneされたchild processを開始
    flags.insert(CloneFlags::CLONE_NEWIPC);
    //新しいnetwork namespaceでcloneされたchild processを開始
    flags.insert(CloneFlags::CLONE_NEWNET);
    //新しいuts namespaceでcloneされたchild processを開始
    flags.insert(CloneFlags::CLONE_NEWUTS);

    //処理が成功したらpid(kernel processの識別番号)を取得
    match clone(
        Box::new(|| child(config.clone())),
        &mut tmp_stack,
        flags,
        Some(Signal::SIGCHLD as i32),
    ) {
        Ok(pid) => Ok(pid),
        Err(err) => {
            error!("{:?}", err);
            Err(Errcode::ChildProcessError(0).into())
        }
    }
}

