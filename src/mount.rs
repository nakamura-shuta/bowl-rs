use crate::errors::Errcode;
use anyhow::{self};
use log::{debug, error};
use std::path::PathBuf;

use nix::mount::{mount, umount2, MntFlags, MsFlags};
use nix::unistd::{chdir, pivot_root};
use std::fs::create_dir_all;
use std::fs::remove_dir;

/// create random directory name
pub fn random_string(mut n: usize) -> String {
    if n == 0 {
        n = 3;
    }
    let dir_name = match random_word::gen_len(n) {
        Some(dir_name) => dir_name,
        None => {
            error!("Generate Directory Name Error");
            "hoge"
        }
    };
    dir_name.to_owned()
}

/// Unmount the "old root".
/// so that included applications cannot access
/// the entire file system
pub fn unmount_path(path: &PathBuf) -> anyhow::Result<()> {
    //Remove the (top-level) filesystem mounted on the path
    match umount2(path, MntFlags::MNT_DETACH) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Unable to umount {}: {}", path.to_str().unwrap(), e);
            Err(Errcode::MountError(0).into())
        }
    }
}

/// Delete the Directory.
/// so that included applications cannot access
/// the entire file system
pub fn delete_dir(path: &PathBuf) -> anyhow::Result<()> {
    match remove_dir(path.as_path()) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!(
                "Unable to delete directory {}: {}",
                path.to_str().unwrap(),
                e
            );
            Err(Errcode::MountError(1).into())
        }
    }
}

/// Create new directory for mount.
pub fn create_directory(path: &PathBuf) -> anyhow::Result<()> {
    match create_dir_all(path) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!(
                "Unable to create directory {}: {}",
                path.to_str().unwrap(),
                e
            );
            Err(Errcode::MountError(2).into())
        }
    }
}

/// Function that is a mount_directory wrapper for syscall
/// Remount the filesystem root "/" using the flag "MS_PRIVATE"
/// which prevents the mount operation from being propagated.
/// see : https://lwn.net/Articles/689856/
pub fn mount_directory(
    path: Option<&PathBuf>,
    mount_point: &PathBuf,
    flags: Vec<MsFlags>,
) -> anyhow::Result<()> {
    let mut ms_flags = MsFlags::empty();
    for flag in flags.iter() {
        ms_flags.insert(*flag);
    }

    match mount::<PathBuf, PathBuf, PathBuf, PathBuf>(path, mount_point, None, ms_flags, None) {
        Ok(_) => Ok(()),
        Err(e) => {
            if let Some(p) = path {
                error!(
                    "Unable to mount {} to {}: {}",
                    p.to_str().unwrap(),
                    mount_point.to_str().unwrap(),
                    e
                );
            } else {
                log::error!("Unable to remount {}: {}", mount_point.to_str().unwrap(), e);
            }
            Err(Errcode::MountError(3).into())
        }
    }
}

/// Changing a Container's Mount Point.
/// 1.Mount the system root in /container
/// 2.Create a new temporary directory
/// 3.Mount a user-specified directory in the temporary directory
/// 4.Perform a root pivot on the two mounted directories
/// 5.Unmount and delete unneeded directories
/// see : https://man7.org/linux/man-pages/man7/mount_namespaces.7.html
pub fn set_mount_point(_mount_directory: &PathBuf) -> anyhow::Result<()> {
    debug!("Setting mount points ...");

    // 1.Mount the system root in /container
    mount_directory(
        None,
        &PathBuf::from("/"),
        vec![MsFlags::MS_REC, MsFlags::MS_PRIVATE],
    )?;
    let new_root = PathBuf::from(format!("/tmp/bowl.{}", random_string(12)));
    debug!(
        "Mount temp directory {}",
        new_root.as_path().to_str().unwrap()
    );

    // 2.Create a new temporary directory
    create_directory(&new_root)?;

    // 3.Mount a user-specified directory in the temporary directory
    mount_directory(
        Some(_mount_directory),
        &new_root,
        vec![MsFlags::MS_BIND, MsFlags::MS_PRIVATE],
    )?;

    // 4.Perform a root pivot on the two mounted directories
    debug!("Pivoting root");
    let old_root_tail = format!("oldroot.{}", random_string(6));
    let put_old = new_root.join(PathBuf::from(old_root_tail.clone()));
    create_directory(&put_old)?;
    if pivot_root(&new_root, &put_old).is_err() {
        return Err(Errcode::MountError(4).into());
    }

    // 5.Unmount and delete unneeded directories
    debug!("Unmounting old root");
    let old_root = PathBuf::from(format!("/{}", old_root_tail));
    if chdir(&PathBuf::from("/")).is_err() {
        return Err(Errcode::MountError(5).into());
    }

    unmount_path(&old_root)?;
    delete_dir(&old_root)?;

    Ok(())
}

/// Clean mount.
/// Placeholder for when you need it someday
pub fn clean_mount(_rootpath: &PathBuf) -> Result<(), Errcode> {
    //unmount_path(&rootpath)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_string_success() {
        let dir_name = random_string(3);
        assert_eq!(dir_name.len(), 3);
    }

    #[test]
    fn random_string_zero() {
        let dir_name = random_string(0);
        assert_eq!(dir_name.len(), 3);
    }
}
