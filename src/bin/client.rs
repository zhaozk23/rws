mod config;
use config::*;
use rustws::WebSocket;
use std::net::TcpStream;

fn main() {
    let addr = format!("{}:{}", HOST, PORT);
    let stream = TcpStream::connect(addr).expect("Couldn't connect to server");
    let mut ws = WebSocket::<TcpStream, CHUNK_SIZE>::new_client(stream);
    ws.client_handshake(HOST).unwrap();
    let message = ws.read().unwrap();
    let message = String::from_utf8(message).unwrap();
    println!("Message from server: {message}");
}
