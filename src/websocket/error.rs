use std::io;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum WsError {
    #[error("client handshake error")]
    ClientHandshakeError,
    #[error("client handshake bad response")]
    ClientHandshakeBadResponse,
    #[error("bad websocket accept")]
    ClientHandshakeBadAccept,
    #[error("No Websocket Accept found in response")]
    ClientHandshakeNoAccept,
    #[error("Multiple Websocket Accepts found in response")]
    ClientHandshakeMultipleAccepts,
    #[error("server handshake error")]
    ServerHandshakeError,
    #[error("server handshake bad request")]
    ServerHandshakeBadRequest,
    #[error("No Websocket Key found in request")]
    ServerHandshakeNoKey,
    #[error("Multiple Websocket Keys found in request")]
    ServerHandshakeMultipleKeys,
    #[error("socket error")]
    SocketError(#[from] io::Error),
    #[error("allocator error")]
    AllocatorError,
    #[error("server close error")]
    ServerCloseError,
    #[error("invalid opcode")]
    InvalidOpcode,
}

pub type Result<T> = std::result::Result<T, WsError>;
