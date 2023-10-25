use std::collections::HashMap;
use std::io::{Read, Write};

use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use json::object;
use log::info;

use crate::inbound_server::{AsyncMessage, InboundMessage, InboundServer};

const LOCAL_ADDR: &str = "127.0.0.1:80";
const REQ_BUFFER_SIZE: usize = 1024;

pub struct InboundHttpServer {
    tx: Sender<AsyncMessage<InboundMessage>>,
}

/// Handler for incoming data on a TCP connection.
/// 
/// Reads the message from the raw ``TCPStream`` and passes the parsed message
/// into the channel specified by ``tx``.
fn handle_connection(mut stream: TcpStream, tx: Sender<AsyncMessage<InboundMessage>>) {
    let mut buffer = [0; REQ_BUFFER_SIZE];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let buffer = Vec::from(&buffer[0..bytes_read]);

    let msg = parse_request(buffer);
    let response = match msg {
        Some(msg) => {
            let (msg, rx) = AsyncMessage::new(msg);

            tx.send(msg).unwrap();

            let result = rx.recv().unwrap();

            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                result.len(),
                result
            )
        }
        None => {
            let result = object! {
                "status" => "failed",
                "error" => "bad request"
            }
            .to_string();

            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                result.len(),
                result
            )
        }
    };

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

/// Parses binary read from a TCPStream into an InboundMessage or None, in case
/// there were any parsing errors.
fn parse_request(bytes: Vec<u8>) -> Option<InboundMessage> {
    let request = String::from_utf8_lossy(&bytes[0..bytes.len()]).to_string();

    if request.len() == 0 {
        return None;
    }

    //TODO: Stick to spec
    //info!("Handling request: size={}, content:\n {}", bytes.len(), request);
    let map: Option<HashMap<String, String>> = request
        .split("\r\n")
        .next()? //GET line
        .split("?")
        .skip(1)
        .next()?
        .split(" ")
        .next()? //Extract params
        .split("&")
        .into_iter()
        .map(|x| {
            //Split params
            let mut split_iter = x.split("="); //Split each key, value
            let key = split_iter.next()?.to_string();
            let val = split_iter.next()?.to_string();
            return Some((key, val));
        })
        .collect(); //Collect into hashmap

    info!("parsed request: {:?}", map);

    InboundMessage::from_hashmap(&map?)
}

impl InboundServer for InboundHttpServer {
    /// Creates new instance of ``InboundServer`` as well as a receiver for the 
    /// async channel into which incomming messages are getting pushed
    fn new() -> (Receiver<AsyncMessage<InboundMessage>>, Self) {
        info!("Initializing inbound http server...");
        let (tx, rx) = mpsc::channel::<AsyncMessage<InboundMessage>>();

        (rx, InboundHttpServer { tx })
    }

    /// Runs the server loop
    fn run(self) {
        info!("Starting inbound http server...");

        let tx = self.tx.clone();

        thread::spawn(move || {
            let listener = TcpListener::bind(LOCAL_ADDR).expect("Unable to bind to TcpListener!");

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

#[cfg(test)]
mod tests {
    use crate::inbound_http_server::parse_request;

    #[test]
    fn test_parse_place_limit() {
        let request = format!(
            "GET /api?action={}&symbol={}&side={}&price={}&amount={}\
            HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n)",
            "place_limit", "BTC", "bid", "1234", "231"
        )
        .into_bytes();
        parse_request(request);
    }
}
