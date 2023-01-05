use crate::errors::Errcode;

use cgroups_rs::cgroup_builder::CgroupBuilder;
use cgroups_rs::hierarchies::V2;
use cgroups_rs::{MaxValue, CgroupPid};
use rlimit::{setrlimit, Resource};
use nix::unistd::Pid;

use std::fs::{canonicalize, remove_dir};
use std::convert::TryInto;

use anyhow::{self};
use log::{info,debug};


//                       KiB    MiB    Gib
const KMEM_LIMIT: i64 = 1024 * 1024 * 1024;
const MEM_LIMIT: i64 = KMEM_LIMIT;
const MAX_PID: MaxValue = MaxValue::Value(64);
const NOFILE_RLIMIT: u64 = 64;

/// Limit resources in containers
pub fn restrict_resources(hostname: &String, pid: Pid) -> anyhow::Result<()>{
    debug!("Restricting resources for hostname {}", hostname);

    // Cgroups
    let cgs = CgroupBuilder::new(hostname)
        // Allocate less CPU time than other processes
        .cpu().shares(256).done()
        // Limiting the memory usage to 1 GiB
        // The user can limit it to less than this, never increase above 1Gib
        .memory().kernel_memory_limit(KMEM_LIMIT).memory_hard_limit(MEM_LIMIT).done()
        // This process can only create a maximum of 64 child processes
        .pid().maximum_number_of_processes(MAX_PID).done()
        // Give an access priority to block IO lower than the system
        .blkio().weight(50).done()
        .build(Box::new(V2::new()));

    // We apply the cgroups rules to the child process we just created
    let pid : u64 = pid.as_raw().try_into().unwrap();
    if let Err(_) = cgs.add_task(CgroupPid::from(pid)) {
        return Err(Errcode::ResourcesError(0).into());
    };

    debug!("resource pid:{}",pid);
    debug!("resource cgs:{:?}",cgs);

    // Rlimit
    // Can create only 64 file descriptors
    if let Err(_) = setrlimit(Resource::NOFILE, NOFILE_RLIMIT, NOFILE_RLIMIT){
        return Err(Errcode::ResourcesError(1).into());
    }

    Ok(())
}

/// Clear all added cgroups restrictions.
pub fn clean_cgroups(hostname: &String) -> anyhow::Result<()>{
    debug!("cleanup cgroups");
    //cgroups v2 は /sys/fs/cgroup/<groupname>/ の
    //下のディレクトリで一元管理しているので、これを削除するだけ
    //canonicalize = file pathを正規化する
    match canonicalize(format!("/sys/fs/cgroup/{}/", hostname)){
        Ok(d) => {
            if let Err(_) = remove_dir(d) {
                return Err(Errcode::ResourcesError(2).into());
            }
        },
        Err(e) => {
            log::error!("Error while canonicalize path: {}", e);
            return Err(Errcode::ResourcesError(3).into());
        }
    }
    Ok(())
}
