use crate::errors::Errcode;

use nix::sys::socket::{recv, send, socketpair, AddressFamily, MsgFlags, SockFlag, SockType};
use std::os::unix::io::RawFd;

use anyhow::{self};

/// Create Sockets
pub fn create_sockets() -> anyhow::Result<(RawFd, RawFd)> {
    match socketpair(
        AddressFamily::Unix,
        SockType::SeqPacket,
        None,
        SockFlag::SOCK_CLOEXEC,
    ) {
        Ok(res) => Ok(res),
        Err(_) => Err(Errcode::SocketError(0).into()),
    }
}
