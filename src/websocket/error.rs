use std::io;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum WsError {
    #[error("client handshake error")]
    ClientHandshakeError,
    #[error("socket error")]
    SocketError(#[from] io::Error),
    #[error("allocator error")]
    AllocatorError,
    #[error("server close error")]
    ServerCloseError,
}

pub type Result<T> = std::result::Result<T, WsError>;