use crate::errors::Errcode;
use crate::ipc::{recv_boolean, send_boolean};

use nix::sched::{unshare, CloneFlags};
use nix::unistd::{setgroups, setresgid, setresuid};
use nix::unistd::{Gid, Pid, Uid};
use std::fs::File;
use std::io::Write;
use std::os::unix::io::RawFd;

use anyhow::{self};
use log::{debug, info};

///setup user namespace with UID
pub fn user_namespace(fd: RawFd, uid: u32) -> anyhow::Result<()> {
    ///ユーザー名前空間の共有を解除して、
    ///呼び出し元のプロセスが既存のプロセスと共有されていない
    ///新しいユーザー名前空間に移動.
    ///see:https://man7.org/linux/man-pages/man2/unshare.2.html
    debug!("setup user namespace with UID {}", uid);
    let has_userns = match unshare(CloneFlags::CLONE_NEWUSER) {
        Ok(_) => true,
        Err(_) => false,
    };
    send_boolean(fd, has_userns)?;

    if recv_boolean(fd)? {
        return Err(Errcode::NamespaceError(0).into());
    }

    if has_userns {
        info!("User namespaces set up");
    } else {
        info!("User namespaces not supported, continuing...");
    }

    debug!("Switching to uid {} / gid {}...", uid, uid);
    let gid = Gid::from_raw(uid);
    let uid = Uid::from_raw(uid);

    //setgroupsを使ってプロセスが属するグループのリストを設定.
    //ここではプロセスのGIDを追加します。
    //※GID:group name.１人のユーザが複数のグループに属することもある
    //see:https://man7.org/linux/man-pages/man2/getgroups.2.html
    if setgroups(&[gid]).is_err() {
        return Err(Errcode::NamespaceError(1).into());
    }

    //setresuidとsetresgidでプロセスのUID,GIDを設定.
    //これで real user ID, effective user ID,保存されたuser IDが設定される.
    //real user ID:あなたが誰であるか(あなたがログインした人) であり、
    //the effective user ID:自分が誰であるかを主張するもの(sudoで一時的に特権を与えたりするときなど)
    //保存されたuser ID：あなたが以前誰であったかを示す
    ///
    //そのため、隔離された環境で、封じ込められたプロセスを root にすることができ、
    //システムによって実際の UID >10000 にマップされ、
    //親システムを汚染することなくユーザーとグループを管理できます。
    //see : https://stackoverflow.com/questions/32455684/difference-between-real-user-id-effective-user-id-and-saved-user-id/32456814#32456814
    if setresgid(gid, gid, gid).is_err() {
        return Err(Errcode::NamespaceError(2).into());
    }

    if setresuid(uid, uid, uid).is_err() {
        return Err(Errcode::NamespaceError(3).into());
    }

    Ok(())
}

///Linux カーネルは、/proc/<pid>/uidmap というファイルを用いて、
//プロセスの名前空間内外のユーザ ID をマッピングしている。
///書式 [ID-inside-ns ID-outside-ns length]
///
////proc/<pid>/uidmap ファイルに
// 0 1000 5
///が含まれている場合、コンテナ内で UID 0 を持つユーザーは、
///コンテナの外では UID 1000 を持つ。
///同様に、内部で1のUIDは外部で1001のUIDにマップされる.
///※内部で6のUIDは外部で1006にマップされない
///see:https://www.cbtnuggets.com/blog/technology/system-admin/linux-file-permission-uid-vs-gid
///
///UIDとGIDを10000以上マッピングしているのは、
///既存のidと衝突しないようにするため.
///UIDは最大2000個までマッピングされる.
///これから再開すると、含まれるプロセス (PIDで一致) が UID 0 を持つと主張する
///(あるいは自分自身を設定する) 場合、カーネルはそれを 10000 の UID で見ることになります。
///GIDについても同じ.

const USERNS_OFFSET: u64 = 10000;
const USERNS_COUNT: u64 = 2000;

pub fn handle_child_uid_map(pid: Pid, fd: RawFd) -> anyhow::Result<()> {
    if recv_boolean(fd)? {
        if let Ok(mut uid_map) = File::create(format!("/proc/{}/{}", pid.as_raw(), "uid_map")) {
            if let Err(_) =
                uid_map.write_all(format!("0 {} {}", USERNS_OFFSET, USERNS_COUNT).as_bytes())
            {
                return Err(Errcode::NamespaceError(4).into());
            }
        } else {
            return Err(Errcode::NamespaceError(5).into());
        }

        if let Ok(mut gid_map) = File::create(format!("/proc/{}/{}", pid.as_raw(), "gid_map")) {
            if let Err(_) =
                gid_map.write_all(format!("0 {} {}", USERNS_OFFSET, USERNS_COUNT).as_bytes())
            {
                return Err(Errcode::NamespaceError(6).into());
            }
        } else {
            return Err(Errcode::NamespaceError(7).into());
        }
    } else {
        info!("No user namespace set up from child process");
    }

    debug!("Child UID/GID map done, sending signal to child to continue...");
    send_boolean(fd, false)
}
