use crate::errors::Errcode;

use log::error;
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

pub fn send_boolean(fd: RawFd, boolean: bool) -> anyhow::Result<()> {
    let data: [u8; 1] = [boolean.into()];
    if let Err(e) = send(fd, &data, MsgFlags::empty()) {
        error!("Cannot send boolean with socket: {:?}", e);
        return Err(Errcode::SocketError(1).into());
    };
    Ok(())
}

pub fn recv_boolean(fd: RawFd) -> anyhow::Result<bool> {
    let mut data: [u8; 1] = [0];
    if let Err(e) = recv(fd, &mut data, MsgFlags::empty()) {
        error!("Cannot receive boolean from socket: {:?}", e);
        return Err(Errcode::SocketError(2).into());
    }
    Ok(data[0] == 1)
}
