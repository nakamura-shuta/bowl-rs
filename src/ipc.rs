use crate::errors::Errcode;

use std::os::unix::io::RawFd;
use nix::sys::socket::{socketpair, AddressFamily, SockType, SockFlag, send, MsgFlags, recv};

use anyhow::{self};

/// Create Sockets
pub fn create_sockets() -> anyhow::Result<(RawFd, RawFd)> {
    match socketpair(
        AddressFamily::Unix,
        SockType::SeqPacket,
        None,
        SockFlag::SOCK_CLOEXEC)
        {
            Ok(res) => Ok(res),
            Err(_) => Err(Errcode::SocketError(0).into())
    }
}
