use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errcode {
    #[error("Invalid Argument : {0}")]
    InvalidArgument(&'static str),

    #[error("Cleanup Error")]
    CleanupFailure(#[from] anyhow::Error),

    #[error("Socket Error")]
    SocketError(u32),
}
