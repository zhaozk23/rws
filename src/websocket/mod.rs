pub mod common;
pub mod error;
pub mod frame;
pub mod message;
pub mod opcode;

use common::{compute_sec_websocket_accept, parse_sec_ws_key};
use error::{Result, WsError};
use frame::Frame;
use message::{Message, MessageKind};
use opcode::Opcode;
use rand;
use std::fmt::Write;
use std::io::{self, Read};
use std::str;

pub struct WebSocket<S, const CHUNK_SIZE: usize> {
    socket: S,
}
// Maybe sending and reading frames shouldn't be public.
impl<S: Read + io::Write, const CHUCK_SIZE: usize> WebSocket<S, CHUCK_SIZE> {
    pub fn new(socket: S) -> Self {
        Self { socket }
    }
    pub fn server_handshake(&mut self) -> Result<()> {
        let mut buffer = [0u8; 1024];
        let buf_size = self.socket.read(&mut buffer).unwrap(); // TODO: work with server io errors
        let request = str::from_utf8(&buffer[0..buf_size]).unwrap(); // TODO: handle utf8 errors (it shouldn't happen in handshake)
        let sec_ws_key = parse_sec_ws_key(request)?;
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
        self.socket.write(handshake.as_bytes()).map_err(|_| {
            eprintln!("Server write handshake to socket failed");
            WsError::ServerHandshakeError
        })?;
        Ok(())
    }
    pub fn client_handshake(&mut self, host: String) -> Result<()> {
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
        self.socket.write(handshake.as_bytes()).map_err(|_| {
            eprintln!("Client write handshake to socket failed");
            WsError::ClientHandshakeError
        })?;
        let mut buffer: [u8; 1024] = [0; 1024];
        let buffer_size = self.socket.read(&mut buffer).map_err(|_| {
            eprintln!("Read from socket failed");
            WsError::ClientHandshakeError
        })?;
        let data = &buffer[..buffer_size];
        if buffer_size < 2 || !data.ends_with(b"\r\n") {
            return Err(WsError::ClientHandshakeError);
        }
        Ok(())
    }
    pub fn send_frame(&mut self, fin: bool, opcode: Opcode, payload: &[u8]) -> Result<()> {
        {
            let mut data = opcode as u8;
            if fin {
                data |= 1 << 7;
            }
            self.socket.write(&[data])?;
        }
        {
            if payload.len() < 126 {
                let data = (1 << 7) | payload.len() as u8;
                self.socket.write(&[data])?;
            } else if payload.len() <= u16::MAX as usize {
                let data = (1 << 7) | 126;
                self.socket.write(&[data])?;
                let len: [u8; 2] = [
                    ((payload.len() >> (8 * 1)) & 0xFF) as u8,
                    ((payload.len() >> (8 * 0)) & 0xFF) as u8,
                ];
                self.socket.write(&len)?;
            } else if payload.len() > u16::MAX as usize {
                let data = (1 << 7) | 127 as u8;
                self.socket.write(&[data])?;
                let len: [u8; 8] = [
                    ((payload.len() >> (8 * 7)) & 0xFF) as u8,
                    ((payload.len() >> (8 * 6)) & 0xFF) as u8,
                    ((payload.len() >> (8 * 5)) & 0xFF) as u8,
                    ((payload.len() >> (8 * 4)) & 0xFF) as u8,
                    ((payload.len() >> (8 * 3)) & 0xFF) as u8,
                    ((payload.len() >> (8 * 2)) & 0xFF) as u8,
                    ((payload.len() >> (8 * 1)) & 0xFF) as u8,
                    ((payload.len() >> (8 * 0)) & 0xFF) as u8,
                ];
                self.socket.write(&len)?;
            }
        }

        let mask: [u8; 4] = rand::random();
        self.socket.write(&mask)?;
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
                self.socket.write(&chunk[0..chunk_size])?;
            }
        }
        Ok(())
    }

    pub fn read_frame(&mut self) -> Result<Frame> {
        let mut header = [0u8; 2];
        self.socket.read(&mut header)?;
        let mut payload_len = 0u64;
        {
            let len = header[1] & 0x7F; // TODO: change this into Header struct
            match len {
                126 => {
                    let mut ext_len = [0u8; 2];
                    self.socket.read(&mut ext_len)?;
                    for i in 0..ext_len.len() {
                        payload_len = (payload_len << 8) | ext_len[i] as u64;
                    }
                }
                127 => {
                    let mut ext_len = [0u8; 8];
                    self.socket.read(&mut ext_len)?;
                    for i in 0..ext_len.len() {
                        payload_len = (payload_len << 8) | ext_len[i] as u64;
                    }
                }
                _ => {
                    payload_len = len as u64;
                }
            }
        }
        {
            let mut mask = [0u8; 4];
            let masked = header[1] >> 7 == 1;
            if masked {
                self.socket.read(&mut mask)?;
            }
        }

        let mut frame = Frame::new();
        frame.fin = header[0] >> 7 == 1;
        frame.opcode = Opcode::try_from(header[0] & 0xF).expect("Invalid opcode");
        frame.payload = vec![0; payload_len as usize];

        if frame.payload.len() > 0 {
            self.socket.read(&mut frame.payload[..])?;
        }

        Ok(frame)
    }
    fn send_message(&mut self, kind: MessageKind, payload: &[u8], chunk_len: usize) -> Result<()> {
        let mut first = true;
        let mut i = 0;
        let total_len = payload.len();
        while i < total_len {
            let len = (total_len - i).min(chunk_len);
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
        }
        Ok(())
    }
    pub fn send_text(&mut self, text: &str) -> Result<()> {
        self.send_message(MessageKind::TEXT, text.as_bytes(), CHUCK_SIZE)
    }
    pub fn send_binary(&mut self, binary: &[u8]) -> Result<()> {
        self.send_message(MessageKind::BIN, binary, CHUCK_SIZE)
    }

    pub fn read_message(&mut self) -> Result<Message> {
        let mut message = Message::new();
        loop {
            let frame = self.read_frame()?;
            if frame.opcode.is_control() {
                match frame.opcode {
                    Opcode::CLOSE => {
                        return Err(WsError::ServerCloseError);
                    }
                    Opcode::PING => {
                        self.send_frame(true, Opcode::PONG, &frame.payload[..])?;
                    }
                    _ => {
                        todo!()
                    }
                }
            } else {
                if message.chunks.is_empty() {
                    message.kind = frame.opcode.into();
                }
                message.chunks.extend_from_slice(&frame.payload[..]);
                if frame.fin {
                    break;
                }
            }
        }
        Ok(message)
    }
}
