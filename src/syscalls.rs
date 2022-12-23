use crate::errors::Errcode;

use syscallz::{Context, Action};

use anyhow::{self};
use log::{info};


pub fn set_syscalls() -> anyhow::Result<()> {
    //TODO
    
    log::debug!("Refusing / Filtering unwanted syscalls");

    // Unconditionnal syscall deny

    // Conditionnal syscall deny

    // Initialize seccomp profile with all syscalls allowed by default
    if let Ok(mut ctx) = Context::init_with_action(Action::Allow) {

        // Configure profile here

        if let Err(_) = ctx.load(){
            return Err(Errcode::SyscallsError(0).into());
        }

        Ok(())
    } else {
        Err(Errcode::SyscallsError(1).into())
    }
}