use crate::websocket::{Result, WsError};
use base64::prelude::*;
use sha1::{Digest, Sha1};
pub(crate) fn compute_sec_websocket_accept(sec_ws_key: String) -> String {
    let mut res = sec_ws_key;
    res.push_str("258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let res = Sha1::digest(res);
    BASE64_STANDARD.encode(res)
}

pub(crate) fn parse_sec_ws_key(request: &str) -> Result<String> {
    for header in request.split("\r\n").skip(1) {
        let header_parts: Vec<&str> = header.splitn(2, ":").collect();
        if header_parts.len() < 2 {
            return Err(WsError::ServerHandshakeError);
        }
        let key = header_parts[0].trim();
        let value = header_parts[1].trim();

        if key == "Sec-WebSocket-Key" {
            return Ok(String::from(value));
        }
    }
    return Err(WsError::ServerHandshakeError);
}
