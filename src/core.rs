use log::info;
use crate::inbound_server::{InboundServer, InboundMessage};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool};
use std::collections::HashMap;
use crate::orderbook::Orderbook;
use crate::symbol::Symbol;

pub struct ExchangeCore {
    inbound_server: InboundServer,
    inbound_reciever: Receiver<InboundMessage>,
    stop_core: Arc<AtomicBool>,
    orderbooks: HashMap<Symbol, Orderbook>,
}

impl ExchangeCore {
    pub fn new() -> ExchangeCore {
        let (inbound_server, inbound_reciever) = InboundServer::new();
        let mut orderbooks = HashMap::new();

        orderbooks.insert(Symbol::BTC, Orderbook::new(Symbol::BTC));
        orderbooks.insert(Symbol::ETH, Orderbook::new(Symbol::ETH));

        ExchangeCore {
            inbound_server,
            inbound_reciever,
            stop_core: Arc::new(AtomicBool::new(false)),
            orderbooks
        }
    }

    pub fn run(mut self) {
        self.inbound_server.run(self.stop_core.clone());

        let mut order_id = 0u64;

        loop {
            if let Ok(msg) = self.inbound_reciever.try_recv() {
                info!("Processing inbound message: {:?}...", msg);
                self.orderbooks.get_mut(&msg.symbol).expect("Orderbook for Symbol not found!")
                    .process_limit(&order_id, msg.side, &msg.limit_price, &msg.amount);
                order_id += 1;
            }
        }
    }

    fn process_inbound_message(self, inbound_message: InboundMessage) {

    }
}