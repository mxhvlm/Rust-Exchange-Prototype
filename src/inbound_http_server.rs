use crate::inbound_server::InboundServer;
use std::sync::mpsc::Receiver;
use crate::inbound_msg::InboundMessage;
use std::sync::mpsc;
use log::info;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

const LOCAL_ADDR: &str = "127.0.0.1:80";
const REQ_BUFFER_SIZE: u32 = 1024;

pub struct InboundHttpServer {

}

impl InboundHttpServer {
    pub fn handle_connection(&self, mut stream: TcpStream) {
        let mut buffer = [0; REQ_BUFFER_SIZE];
        let bytes_read = stream.read(&mut buffer).unwrap();
        info!("Handling request: size={}, content:\n {}",bytes_read, String::from_utf8_lossy(&buffer[0..bytes_read]));

        let response = "HTTP/1.1 200 OK\r\n\r\n";

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

impl InboundServer for InboundHttpServer {
    fn new() -> (Receiver<InboundMessage>, Self) {
        info!("Initializing inbound http server...");
        let (tx, rx) = mpsc::channel::<InboundMessage>();

        (rx, InboundHttpServer {})
    }

    fn run(self) {
        info!("Starting inbound http server...");

        let listener = TcpListener::bind(LOCAL_ADDR).expect("Unable to bind to \
            TcpListener!");

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            self.handle_connection(stream);
        }
    }
}