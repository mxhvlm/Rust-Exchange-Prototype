use log::info;
use crate::inbound_server::{InboundServer, InboundMessage};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool};

pub struct ExchangeCore {
    inbound_server: InboundServer,
    inbound_reciever: Receiver<InboundMessage>,
    stop_core: Arc<AtomicBool>,
}

impl ExchangeCore {
    pub fn new() -> ExchangeCore {
        let (inbound_server, inbound_reciever) = InboundServer::new();
        ExchangeCore {
            inbound_server,
            inbound_reciever,
            stop_core: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn run(self) {
        self.inbound_server.run(self.stop_core.clone());

        loop {
            if let Ok(msg) = self.inbound_reciever.try_recv() {
                info!("Processing inbound message: {:?}...", msg);
            }
        }
    }

    fn process_inbound_message(self, inbound_message: InboundMessage) {

    }
}