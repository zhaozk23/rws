use rustws::WebSocket;
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
        if let Err(e) = ws.send_text("Hello World") {
            println!("ERROR: send message failed: {e}");
            continue;
        }
    }
}
