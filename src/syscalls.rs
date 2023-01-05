use crate::errors::Errcode;

use libc::TIOCSTI;
use nix::sched::CloneFlags;
use nix::sys::stat::Mode;
use syscallz::{Action, Cmp, Comparator, Context, Syscall};

use anyhow::{self};
use log::{debug};

//operation not permitted error
const EPERM: u16 = 1;

pub fn set_syscalls() -> anyhow::Result<()> {
    debug!("Refusing / Filtering unwanted syscalls");
    let s_isuid: u64 = Mode::S_ISUID.bits().into();
    let s_isgid: u64 = Mode::S_ISGID.bits().into();
    let clone_new_user: u64 = CloneFlags::CLONE_NEWUSER.bits() as u64;

    // Unconditionnal syscall deny
    let syscalls_reject = [
        Syscall::keyctl,
        Syscall::add_key,
        Syscall::request_key,
        Syscall::mbind,
        Syscall::migrate_pages,
        Syscall::move_pages,
        Syscall::set_mempolicy,
        Syscall::userfaultfd,
        Syscall::perf_event_open,
    ];

    // Conditionnal syscall deny
    let syscalls_reject_conditional = [
        (Syscall::chmod, 1, s_isuid),
        (Syscall::chmod, 1, s_isgid),
        (Syscall::fchmod, 1, s_isuid),
        (Syscall::fchmod, 1, s_isgid),
        (Syscall::fchmodat, 2, s_isuid),
        (Syscall::fchmodat, 2, s_isgid),
        (Syscall::unshare, 0, clone_new_user),
        (Syscall::clone, 0, clone_new_user),
        (Syscall::ioctl, 1, TIOCSTI),
    ];

    // Initialize seccomp profile with all syscalls allowed by default
    if let Ok(mut ctx) = Context::init_with_action(Action::Allow) {
        for (sc, ind, biteq) in syscalls_reject_conditional.iter() {
            reject_conditional_syscall(&mut ctx, *ind, sc, *biteq)?;
        }

        for sc in syscalls_reject.iter() {
            reject_syscall(&mut ctx, sc)?;
        }

        if ctx.load().is_err() {
            return Err(Errcode::SyscallsError(0).into());
        }

        Ok(())
    } else {
        Err(Errcode::SyscallsError(1).into())
    }
}

/// Restricting Unconditional System Calls
/// Reject system calls that you do not want your children to execute.
fn reject_syscall(ctx: &mut Context, sc: &Syscall) -> anyhow::Result<()> {
    match ctx.set_action_for_syscall(Action::Errno(EPERM), *sc) {
        Ok(_) => Ok(()),
        Err(_) => Err(Errcode::SyscallsError(3).into()),
    }
}

/// Restricting Conditional System Calls.
/// You can restrict a syscall if certain conditions are met.
/// To do this, create a rule that takes a value and returns whether or not permission should be set.
fn reject_conditional_syscall(
    ctx: &mut Context,
    ind: u32,
    sc: &Syscall,
    biteq: u64,
) -> anyhow::Result<()> {
    //システムコールに渡された引数番号ind,biteqを取得し、マスクを使用して値と比較する
    match ctx.set_rule_for_syscall(
        Action::Errno(EPERM),
        *sc,
        &[Comparator::new(ind, Cmp::MaskedEq, biteq, Some(biteq))],
    ) {
        Ok(_) => Ok(()),
        Err(_) => Err(Errcode::SyscallsError(2).into()),
    }
}
