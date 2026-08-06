use crate::websocket::{Result, WsError};
use base64::prelude::*;
use sha1::{Digest, Sha1};
pub(crate) fn compute_sec_websocket_accept(sec_ws_key: String) -> String {
    let mut res = sec_ws_key;
    res.push_str("258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let res = Sha1::digest(res);
    BASE64_STANDARD.encode(res)
}

pub(crate) fn parse_sec_ws_key(request: &mut &str) -> Result<String> {
    let mut found = false;
    let mut ws_key = String::new();
    const LINE_SEP: &str = "\r\n";
    if let Some(index) = request.find(LINE_SEP) {
        *request = &request[index + LINE_SEP.len()..];
    } else {
        return Err(WsError::ServerHandshakeBadRequest);
    }
    while !request.is_empty() && !request.starts_with(LINE_SEP) {
        let index = request
            .find(LINE_SEP)
            .ok_or(WsError::ServerHandshakeBadRequest)?;
        let header = &request[..index];
        *request = &request[index + LINE_SEP.len()..];
        let header_parts: Vec<&str> = header.splitn(2, ":").collect();
        if header_parts.len() < 2 {
            return Err(WsError::ServerHandshakeError);
        }
        let key = header_parts[0].trim();
        let value = header_parts[1].trim();

        if key == "Sec-WebSocket-Key" {
            if found {
                return Err(WsError::ServerHandshakeMultipleKeys);
            }
            ws_key = value.to_string();
            found = true;
        }
    }
    if !request.starts_with(LINE_SEP) {
        return Err(WsError::ServerHandshakeBadRequest);
    }
    *request = &request[LINE_SEP.len()..];
    if !found {
        return Err(WsError::ServerHandshakeNoKey);
    }
    Ok(ws_key)
}

pub(crate) fn parse_sec_ws_accept(response: &mut &str) -> Result<String> {
    let mut found = false;
    let mut ws_accept = String::new();
    const LINE_SEP: &str = "\r\n";
    if let Some(index) = response.find(LINE_SEP) {
        *response = &response[index + LINE_SEP.len()..];
    } else {
        return Err(WsError::ClientHandshakeBadResponse);
    }
    while !response.is_empty() && !response.starts_with(LINE_SEP) {
        let index = response
            .find(LINE_SEP)
            .ok_or(WsError::ClientHandshakeBadResponse)?;
        let header = &response[..index];
        *response = &response[index + LINE_SEP.len()..];
        let header_parts: Vec<&str> = header.splitn(2, ":").collect();
        if header_parts.len() < 2 {
            return Err(WsError::ClientHandshakeBadResponse);
        }
        let key = header_parts[0].trim();
        let value = header_parts[1].trim();

        if key == "Sec-WebSocket-Accept" {
            if found {
                return Err(WsError::ClientHandshakeMultipleAccepts);
            }
            ws_accept = value.to_string();
            found = true;
        }
    }
    if !response.starts_with(LINE_SEP) {
        return Err(WsError::ClientHandshakeBadResponse);
    }
    *response = &response[LINE_SEP.len()..];
    if !found {
        return Err(WsError::ClientHandshakeNoAccept);
    }
    Ok(ws_accept)
}
