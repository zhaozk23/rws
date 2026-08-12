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
            return Err(WsError::ServerHandshakeBadRequest);
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

pub(crate) fn verify_utf8(buf: &[u8], pos: usize) -> Result<Option<usize>> {
    let rem = match buf.get(pos..) {
        Some(r) if !r.is_empty() => r,
        _ => return Ok(None),
    };
    let b0 = rem[0];
    let min_len = if b0 < 0x80 {
        // 1字节:成功
        return Ok(Some(1));
    } else if b0 & 0xE0 == 0xC0 {
        // 110xxxxx => 2字节
        2
    } else if b0 & 0xF0 == 0xE0 {
        // 1110xxxx => 3字节
        3
    } else if b0 & 0xF8 == 0xF0 {
        // 11110xxx => 4字节
        4
    } else {
        // 10xxxxxx(孤立续字节)或非法 => 错
        return Err(WsError::InvalidUtf8);
    };
    if rem.len() < min_len {
        return Ok(None);
    };
    let mut cp = (b0
        & match min_len {
            2 => 0x1F,
            3 => 0x0F,
            _ => 0x07,
        }) as u32;
    for &c in rem.iter().take(min_len).skip(1) {
        if c & 0xC0 != 0x80 {
            return Err(WsError::InvalidUtf8);
        }
        cp = (cp << 6) | (c & 0x3F) as u32;
    }
    if cp
        < match min_len {
            2 => 0x80,
            3 => 0x800,
            _ => 0x10000,
        }
        || (0xD800..=0xDFFF).contains(&cp)
        || cp > 0x10FFFF
    {
        return Err(WsError::InvalidUtf8);
    }
    Ok(Some(min_len))
}

pub(crate) fn extend_utf8(buf: &mut Vec<u8>, pos: usize) {
    let c = buf[pos];
    let size = if c & 0x80 == 0 {
        1
    } else if c & 0xE0 == 0xC0 {
        2
    } else if c & 0xF0 == 0xE0 {
        3
    } else {
        4
    };
    while buf.len() - pos < size {
        buf.push(0b1000_0000);
    }
}
