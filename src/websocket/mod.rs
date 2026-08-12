pub mod common;
pub mod error;
pub mod frame;
pub mod message;
pub mod opcode;

use common::{
    compute_sec_websocket_accept, extend_utf8, parse_sec_ws_accept, parse_sec_ws_key, verify_utf8,
};
use error::{Result, WsError};
use frame::FrameHeader;
use message::{Message, MessageKind};
use opcode::Opcode;
use rand;
use std::fmt::Write;
use std::io::{self, BufRead, BufReader, Read};
use std::str;

pub struct WebSocket<S, const CHUNK_SIZE: usize> {
    socket: BufReader<S>,
    is_client: bool,
}

impl<S: Read + io::Write, const CHUNK_SIZE: usize> WebSocket<S, CHUNK_SIZE> {
    pub fn new_server(socket: S) -> Self {
        Self {
            socket: BufReader::new(socket),
            is_client: false,
        }
    }
    pub fn new_client(socket: S) -> Self {
        Self {
            socket: BufReader::new(socket),
            is_client: true,
        }
    }
    pub fn server_handshake(&mut self) -> Result<()> {
        let sec_ws_key = {
            let mut buffer = Vec::with_capacity(1024);
            loop {
                let n = self.socket.read_until(b'\n', &mut buffer)?;
                if n == 0 {
                    // EOF
                    return Err(WsError::ServerHandshakeBadRequest);
                }
                if buffer.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            let mut request =
                str::from_utf8(&buffer).map_err(|_| WsError::ServerHandshakeBadRequest)?;
            parse_sec_ws_key(&mut request)?
        };
        let mut handshake = String::with_capacity(1024);
        handshake.push_str("HTTP/1.1 101 Switching Protocols\r\n");
        handshake.push_str("Upgrade: websocket\r\n");
        handshake.push_str("Connection: Upgrade\r\n");
        write!(
            &mut handshake,
            "Sec-WebSocket-Accept: {}\r\n",
            compute_sec_websocket_accept(sec_ws_key)
        )
        .map_err(|_| {
            eprintln!("Write key to handshake failed");
            WsError::ServerHandshakeError
        })?;
        handshake.push_str("\r\n");
        self.socket
            .get_mut()
            .write_all(handshake.as_bytes())
            .map_err(|_| {
                eprintln!("Server write handshake to socket failed");
                WsError::ServerHandshakeError
            })?;
        Ok(())
    }
    pub fn client_handshake(&mut self, host: &str) -> Result<()> {
        let mut handshake = String::with_capacity(1024);
        handshake.push_str("GET / HTTP/1.1\r\n");
        write!(&mut handshake, "Host: {}\r\n", host).map_err(|_| {
            eprintln!("Write host: {host} to handshake failed");
            WsError::ClientHandshakeError
        })?;
        handshake.push_str("Upgrade: websocket\r\n");
        handshake.push_str("Connection: Upgrade\r\n");
        handshake.push_str("Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n");
        handshake.push_str("Sec-WebSocket-Version: 13\r\n");
        handshake.push_str("\r\n");
        self.socket
            .get_mut()
            .write_all(handshake.as_bytes())
            .map_err(|_| {
                eprintln!("Client write handshake to socket failed");
                WsError::ClientHandshakeError
            })?;
        let sec_ws_accept = {
            let mut buffer = Vec::with_capacity(1024);
            loop {
                let n = self.socket.read_until(b'\n', &mut buffer)?;
                if n == 0 {
                    // EOF
                    return Err(WsError::ClientHandshakeBadResponse);
                }
                if buffer.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            let mut response =
                str::from_utf8(&buffer).map_err(|_| WsError::ClientHandshakeBadResponse)?;
            parse_sec_ws_accept(&mut response)?
        };
        if sec_ws_accept != "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=" {
            return Err(WsError::ClientHandshakeBadAccept);
        }
        Ok(())
    }
    pub fn send_frame(&mut self, fin: bool, opcode: Opcode, payload: &[u8]) -> Result<()> {
        if opcode.is_control() && (payload.len() > 125 || !fin) {
            return Err(WsError::ControlFrameTooBig);
        }
        let socket = self.socket.get_mut();
        {
            let mut data = opcode as u8;
            if fin {
                data |= 1 << 7;
            }
            socket.write_all(&[data])?;
        }
        {
            let mask_bit: u8 = if self.is_client { 1 << 7 } else { 0 };
            if payload.len() < 126 {
                let data = mask_bit | payload.len() as u8;
                socket.write_all(&[data])?;
            } else if payload.len() <= u16::MAX as usize {
                let data = mask_bit | 126;
                socket.write_all(&[data])?;
                let len = (payload.len() as u16).to_be_bytes();
                socket.write_all(&len)?;
            } else if payload.len() > u16::MAX as usize {
                let data = mask_bit | 127u8;
                socket.write_all(&[data])?;
                let len = payload.len().to_be_bytes();
                socket.write_all(&len)?;
            }
        }
        if self.is_client {
            let mask: [u8; 4] = rand::random();
            socket.write_all(&mask)?;
            {
                let mut i = 0;
                while i < payload.len() {
                    let mut chunk = [0u8; 1024];
                    let mut chunk_size = 0;
                    while i < payload.len() && chunk_size < chunk.len() {
                        chunk[chunk_size] = payload[i] ^ mask[i % 4];
                        chunk_size += 1;
                        i += 1;
                    }
                    socket.write_all(&chunk[0..chunk_size])?;
                }
            }
        } else {
            socket.write_all(payload)?;
        }
        Ok(())
    }

    fn read_frame_header(&mut self) -> Result<FrameHeader> {
        let mut header = [0u8; 2];
        self.socket.read_exact(&mut header)?;
        let mut frame_header = FrameHeader::new();
        frame_header.fin = (header[0] >> 7) & 1 == 1;
        frame_header.rsv1 = (header[0] >> 6) & 1 == 1;
        frame_header.rsv2 = (header[0] >> 5) & 1 == 1;
        frame_header.rsv3 = (header[0] >> 4) & 1 == 1;
        frame_header.opcode =
            Opcode::try_from(header[0] & 0xF).map_err(|_| WsError::InvalidOpcode)?;
        frame_header.masked = (header[1] >> 7) & 1 == 1;

        {
            let len = header[1] & 0x7F;
            match len {
                126 => {
                    let mut ext_len = [0u8; 2];
                    self.socket.read_exact(&mut ext_len)?;
                    frame_header.payload_len = u16::from_be_bytes(ext_len) as usize;
                }
                127 => {
                    let mut ext_len = [0u8; 8];
                    self.socket.read_exact(&mut ext_len)?;
                    frame_header.payload_len = u64::from_be_bytes(ext_len) as usize;
                }
                _ => {
                    frame_header.payload_len = len as usize;
                }
            }
        }
        if frame_header.opcode.is_control() && (frame_header.payload_len > 125 || !frame_header.fin)
        {
            return Err(WsError::ControlFrameTooBig);
        }

        if frame_header.rsv1 || frame_header.rsv2 || frame_header.rsv3 {
            return Err(WsError::ReservedBitsNotNegotiated);
        }

        if frame_header.masked {
            self.socket.read_exact(&mut frame_header.mask)?;
        }

        Ok(frame_header)
    }

    fn read_frame_payload(&mut self, frame_header: &FrameHeader) -> Result<Vec<u8>> {
        let mut payload = vec![0; frame_header.payload_len];
        let mut payload_size = 0;
        while payload_size < payload.len() {
            payload_size +=
                self.read_frame_payload_chunk(frame_header, &mut payload, payload_size)?;
        }
        Ok(payload)
    }

    fn read_frame_payload_chunk(
        &mut self,
        frame_header: &FrameHeader,
        buf: &mut [u8],
        payload_size: usize,
    ) -> Result<usize> {
        assert_eq!(buf.len(), frame_header.payload_len);
        if payload_size >= frame_header.payload_len {
            return Ok(0);
        }
        let start = payload_size;
        let n = self.socket.read(&mut buf[start..])?;
        if frame_header.masked {
            for i in 0..n {
                buf[start + i] ^= frame_header.mask[(start + i) % 4];
            }
        }
        Ok(n)
    }

    pub fn send_message(&mut self, kind: MessageKind, payload: &[u8]) -> Result<()> {
        let mut first = true;
        let mut i = 0;
        let total_len = payload.len();
        loop {
            let len = (total_len - i).min(CHUNK_SIZE);
            let opcode = if first {
                match kind {
                    MessageKind::BIN => Opcode::BIN,
                    MessageKind::TEXT => Opcode::TEXT,
                }
            } else {
                Opcode::CONT
            };
            self.send_frame(i + len == total_len, opcode, &payload[i..i + len])?;
            i += len;
            first = false;
            if i >= total_len {
                break;
            }
        }
        Ok(())
    }
    pub fn send_text(&mut self, payload: &str) -> Result<()> {
        self.send_message(MessageKind::TEXT, payload.as_bytes())
    }
    pub fn send_binary(&mut self, payload: &[u8]) -> Result<()> {
        self.send_message(MessageKind::BIN, payload)
    }
    pub fn read_message(&mut self) -> Result<Message> {
        let mut message = Message::new();
        let mut cont = false;
        let mut verify_pos = 0;
        loop {
            let frame_header = self.read_frame_header()?;
            if frame_header.opcode.is_control() {
                match frame_header.opcode {
                    Opcode::CLOSE => {
                        return Err(WsError::CloseFrameSent);
                    }
                    Opcode::PING => {
                        let ping_payload = self.read_frame_payload(&frame_header)?;
                        self.send_frame(true, Opcode::PONG, &ping_payload)?;
                    }
                    _ => {
                        let _ = self.read_frame_payload(&frame_header)?;
                    }
                }
            } else {
                if !cont {
                    message.kind = frame_header.opcode.try_into()?;
                    cont = true;
                } else if frame_header.opcode != Opcode::CONT {
                    return Err(WsError::UnexpectedOpcode);
                }
                let mut frame_payload = vec![0; frame_header.payload_len];
                let mut frame_payload_size = 0;
                while frame_payload_size < frame_payload.len() {
                    let n = self.read_frame_payload_chunk(
                        &frame_header,
                        &mut frame_payload,
                        frame_payload_size,
                    )?;
                    message.payload.extend_from_slice(
                        &frame_payload[frame_payload_size..frame_payload_size + n],
                    );
                    frame_payload_size += n;

                    if message.kind == MessageKind::TEXT {
                        while verify_pos < message.payload.len() {
                            match verify_utf8(&message.payload, verify_pos) {
                                Ok(Some(n)) => verify_pos += n,
                                Ok(None) => {
                                    if frame_header.fin {
                                        return Err(WsError::InvalidUtf8);
                                    }
                                    let saved_len = message.payload.len();
                                    extend_utf8(&mut message.payload, verify_pos);
                                    verify_utf8(&message.payload, verify_pos)?;
                                    message.payload.truncate(saved_len);
                                    break;
                                }
                                Err(e) => return Err(e),
                            }
                        }
                    }
                }
                if frame_header.fin {
                    break;
                }
            }
        }
        Ok(message)
    }
}
