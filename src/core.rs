use log::info;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use crate::inbound_server::InboundServer;
use crate::orderbook::Orderbook;
use crate::symbol::Symbol;
use crate::inbound_msg::InboundMessage;

pub struct ExchangeCore {
    orderbooks: HashMap<Symbol, Orderbook>,
    last_order_id: u64
}

impl ExchangeCore {
    pub fn new() -> ExchangeCore {
        let mut orderbooks = HashMap::new();

        orderbooks.insert(Symbol::BTC, Orderbook::new(Symbol::BTC));
        orderbooks.insert(Symbol::ETH, Orderbook::new(Symbol::ETH));

        ExchangeCore {
            orderbooks,
            last_order_id: 0u64,
        }
    }

    pub fn run(mut self) {
        let (mut inbound_server, inbound_reciever) = InboundServer::new();

        let stop_inbound_server = Arc::new(AtomicBool::new(false));
        inbound_server.run();

        loop {
            if let Ok(msg) = inbound_reciever.try_recv() {
                info!("Processing inbound message: {:?}...", msg);
                self.process_inbound_message(&msg);
            }
        }
    }

    fn process_inbound_message(&mut self, msg: &InboundMessage) {
        self.orderbooks.get_mut(&msg.symbol).expect("Orderbook for Symbol not found!")
            .insert_try_exec_limit(&self.last_order_id, msg.side.clone(), &msg.limit_price, &msg.amount);
        self.last_order_id += 1;
    }
}