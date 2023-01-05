use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errcode {
    #[error("Invalid Argument : {0}")]
    InvalidArgument(&'static str),

    #[error("Cleanup Error")]
    CleanupFailure(#[from] anyhow::Error),

    #[error("Socket Error")]
    SocketError(u32),

    #[error("Container Error")]
    ContainerError(u8),

    #[error("Child Process Error")]
    ChildProcessError(u8),

    #[error("Hostname Error")]
    HostnameError(u8),

    #[error("Generate Word Error")]
    WordError,

    #[error("Mount Error")]
    MountError(u8),

    #[error("Namespace Error")]
    NamespaceError(u8),

    #[error("Capabilitie Error")]
    CapaError(u8),
    
    #[error("Syscalls Error")]
    SyscallsError(u8),

    #[error("Resources Error")]
    ResourcesError(u8),
}
