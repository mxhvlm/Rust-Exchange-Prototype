use std::net::{TcpStream, TcpListener};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::{thread, fmt};
use std::sync::atomic::{AtomicBool, Ordering};
use log::{info, warn};
use std::io::{Read, ErrorKind};
use std::thread::sleep;
use crate::symbol::{Symbol, AskOrBuy};
use std::fmt::Formatter;
use rust_decimal::Decimal;
use rand::{random, Rng};

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

#[derive(Debug)]
pub enum MessageType {
    PLACE_LIMIT_ORDER = 1,
    DELETE_LIMIT_ORDER = 2,
    PLACE_MARKET_ORDER = 3
}

impl MessageType {
    fn from_u8(value: u8) -> Option<MessageType> {
        match value {
            1 => Some(MessageType::PLACE_LIMIT_ORDER),
            2 => Some(MessageType::DELETE_LIMIT_ORDER),
            3 => Some(MessageType::PLACE_MARKET_ORDER),
            _ => None
        }
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct InboundMessage {
    pub message_type: MessageType,
    pub symbol: Symbol,
    pub side: AskOrBuy,
    pub limit_price: Decimal
}

struct Client {
    socket: TcpStream,
    client_num: u32
}

pub struct InboundServer {
    clients: Vec<Client>,
    message_transmitter: Sender<InboundMessage>,
    last_client_num: u32
}

impl InboundMessage {
    pub fn from_bytes(buff: Vec<u8>) -> Result<InboundMessage, ErrorKind> {
        info!("{}", buff.len());
        match buff.len() {
            MSG_SIZE => {
                let mut iter = buff.into_iter();

                let symbol = Symbol::from_u8(iter.next().unwrap()).expect("invlaid symbol");

                let message_type = MessageType::from_u8(iter.next().unwrap()).expect("invalid message num");

                let side = AskOrBuy::from_u8(iter.next().unwrap()).expect("invalid AskOrBuy");

                let limit_price = match message_type {
                    MessageType::PLACE_MARKET_ORDER => Decimal::from(0),
                    _ => Decimal::from(500 + rand::thread_rng().gen_range(0..100)),
                };

                Ok(InboundMessage{symbol, side, message_type, limit_price })
            },
            _ => Err(ErrorKind::InvalidData)
        }
    }
}

impl InboundServer {
    pub fn new() -> (InboundServer, Receiver<InboundMessage>) {
        let (tx, rx) = mpsc::channel::<InboundMessage>();

        (InboundServer {
            clients: Vec::new(),
            message_transmitter: tx,
            last_client_num: 0
        }, rx)
    }

    pub fn run(mut self, stop_server: Arc<AtomicBool>) {
        info!("Starting server on {}", LOCAL);

        thread::spawn(move || {
            let server = TcpListener::bind(LOCAL).expect("Unable to bind TCPListener!"); //TODO: Handle and print error to log
            server.set_nonblocking(true).expect("Failed to set TcpListener to nonblocking!");
            loop {
                if stop_server.load(Ordering::Relaxed) {
                    info!("Stopping server...");
                    break;
                }

                if let Ok((mut socket, addr)) = server.accept() {
                    info!("New connection from: {}", addr);
                    let tx = self.message_transmitter.clone();

                    self.clients.push(
                        Client{socket: socket.try_clone().expect("Failed to clone the client"),
                            client_num: self.last_client_num}
                    );

                    thread::spawn(move || loop {
                        let mut buff = vec![0 as u8; MSG_SIZE];

                        match socket.read(&mut buff) {
                            Ok(_) => {
                                //iterate over buffer and collect every byte into a buffer until we hit null byte
                                //let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                                let msg = InboundMessage::from_bytes(buff);

                                match msg {
                                    Err(err) => warn!("Failed to read inbound message: {:?}", err),
                                    Ok(in_msg) => {
                                        tx.send(in_msg).expect("failed to send msg to rx");
                                    }
                                }
                            },
                            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                            Err(_) => {
                                info!("Closing connection with {}", addr);
                                break;
                            }
                        }
                        //TODO: Write confirmation
                        sleep(std::time::Duration::from_millis(100));
                    });
                }
            }
        });
        // sleep(Duration::from_millis(1000));
    }
}