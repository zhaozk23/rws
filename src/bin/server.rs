use rustws::{WebSocket, opcode::Opcode};
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
        let message = ws.read_message();
        if let Err(e) = message {
            println!("ERROR: could not read message from client: {e}");
            continue;
        }
        let message = message.unwrap();
        println!("INFO: Client sent: {l} bytes", l = message.payload.len());
        if let Err(e) = ws.send_message(message.kind, &message.payload) {
            println!("ERROR: send message failed: {e}");
            continue;
        }
        if let Err(e) = ws.send_frame(true, Opcode::CLOSE, &[]) {
            println!("ERROR: failed to close the connection: {e}");
            continue;
        }
    }
}
