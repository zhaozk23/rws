use rustws::{WebSocket, error::WsError, opcode::Opcode};
use std::net::{TcpListener, TcpStream};
mod config;
use config::*;
fn main() {
    let addr = format!("{}:{}", HOST, PORT);
    let server = TcpListener::bind(addr).unwrap();
    println!("Listening to {}:{}", HOST, PORT);
    for stream in server.incoming() {
        let client = stream.unwrap();
        let mut ws = WebSocket::<TcpStream, CHUNK_SIZE>::new_server(client);
        if let Err(e) = ws.server_handshake() {
            println!("ERROR: handshake failes: {e}");
            continue;
        }
        loop {
            let message = ws.read_message();
            match message {
                Ok(message) => {
                    println!("INFO: Client sent: {l} bytes", l = message.payload.len());
                    if let Err(e) = ws.send_message(message.kind, &message.payload) {
                        println!("ERROR: send message failed: {e}");
                        let _ = ws.send_frame(true, Opcode::CLOSE, &[]);
                        break;
                    }
                }
                Err(WsError::CloseFrameSent) => {
                    println!("INFO: Client closed connection");
                    let _ = ws.send_frame(true, Opcode::CLOSE, &[]);
                    break;
                }
                Err(e) => {
                    println!("ERROR: read failed: {e}");
                    let _ = ws.send_frame(true, Opcode::CLOSE, &[]);
                    break;
                }
            }
        }
    }
}
