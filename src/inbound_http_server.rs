use crate::inbound_server::{InboundServer, AsyncMessage};
use std::sync::mpsc::{Receiver, Sender};
use crate::inbound_msg::InboundMessage;
use std::sync::mpsc;
use log::info;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

const LOCAL_ADDR: &str = "127.0.0.1:80";
const REQ_BUFFER_SIZE: usize = 1024;

pub struct InboundHttpServer {
    pub tx: Sender<AsyncMessage<InboundMessage>>
}

impl InboundHttpServer {

}

pub fn handle_connection(mut stream: TcpStream, tx: Sender<AsyncMessage<InboundMessage>>) {
    let mut buffer = [0; REQ_BUFFER_SIZE];
    let bytes_read = stream.read(&mut buffer).unwrap();
    info!("Handling request: size={}, content:\n {}",bytes_read, String::from_utf8_lossy(&buffer[0..bytes_read]));

    let (msg, rx) = AsyncMessage::new(InboundMessage::get_dummy());

    tx.send(msg);

    let result = match rx.recv().unwrap() {
        Ok(status) => status,
        Err(err) => format!("{:?}", err)
    };

    let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", result.len(), result);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

impl InboundServer for InboundHttpServer {
    fn new() -> (Receiver<AsyncMessage<InboundMessage>>, Self) {
        info!("Initializing inbound http server...");
        let (tx, rx) = mpsc::channel::<AsyncMessage<InboundMessage>>();

        (rx, InboundHttpServer {tx})
    }

    fn run(self) {
        info!("Starting inbound http server...");

        let tx = self.tx.clone();

        thread::spawn(move || {
            let listener = TcpListener::bind(LOCAL_ADDR).expect("Unable to bind to \
            TcpListener!");

            for stream in listener.incoming() {
                let stream = stream.unwrap();
                let tx = tx.clone();
                thread::spawn(move || {
                    handle_connection(stream, tx);
                });
            }
        });
    }
}