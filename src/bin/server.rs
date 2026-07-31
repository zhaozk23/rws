use std::net::{TcpListener, TcpStream};
use rustws::WebSocket;
const HOST: &str = "127.0.0.1";
const PORT: u16 = 6969;
fn main() {
    let addr = format!("{}:{}", HOST, PORT);
    let server = TcpListener::bind(addr).unwrap();
    println!("Listening to {}:{}", HOST, PORT);
    for stream in server.incoming() {
        let client = stream.unwrap();
        let mut ws  = WebSocket::<TcpStream, 1024>::new(client);
        ws.server_handshake().unwrap();
        ws.send_text("Hello World").unwrap();
    }
}
