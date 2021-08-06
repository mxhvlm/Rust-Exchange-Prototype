use crate::inbound_server::InboundServer;
use std::sync::mpsc::Receiver;
use crate::inbound_msg::InboundMessage;
use std::sync::mpsc;
use log::info;

pub struct InboundHttpServer {

}

impl InboundServer for InboundHttpServer {
    fn new() -> (Receiver<InboundMessage>, Self) {
        info!("Initializing inbound http server...");
        let (tx, rx) = mpsc::channel::<InboundMessage>();

        (rx, InboundHttpServer {})
    }

    fn run(self) {
        info!("Starting inbound http server...");
    }
}