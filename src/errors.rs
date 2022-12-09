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
}
