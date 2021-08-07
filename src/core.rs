
use std::collections::HashMap;
use std::io::ErrorKind;


use log::{error, info};

use crate::inbound_http_server::InboundHttpServer;
use crate::inbound_server::{InboundMessage, InboundServer, MessageType};
use crate::orderbook::{Orderbook, InsertLimitResult};
use crate::symbol::Symbol;

pub struct ExchangeCore {
    orderbooks: HashMap<Symbol, Orderbook>,
    orderbook_id_lookup: HashMap<u64, Orderbook>,
    last_order_id: u64,
}

impl ExchangeCore {
    pub fn new() -> ExchangeCore {
        let mut orderbooks = HashMap::new();
        let mut orderbook_id_lookup = HashMap::new();

        orderbooks.insert(Symbol::BTC, Orderbook::new(Symbol::BTC));
        orderbooks.insert(Symbol::ETH, Orderbook::new(Symbol::ETH));

        ExchangeCore {
            orderbooks,
            last_order_id: 0u64,
            orderbook_id_lookup
        }
    }

    pub fn run(mut self) {
        let (inbound_reciever, inbound_server) = InboundHttpServer::new();

        inbound_server.run();

        loop {
            if let Ok(msg) = inbound_reciever.try_recv() {
                let mut cmd = msg.cmd.clone();
                info!("Processing inbound message: {:?}...", &cmd);
                msg.resp
                    .send(self.process_inbound_message(&mut cmd))
                    .unwrap();
            }
        }
    }

    //TODO: find some other way of returning the msg.order_id
    //TODO: handle the way msg.symbol gets checked properly
    fn process_inbound_message(&mut self, msg: &mut InboundMessage) -> String {
        match msg.message_type {
            MessageType::PlaceLimitOrder => {
                match (msg.limit_price, msg.amount, &msg.side, &msg.symbol) {
                    (Some(price), Some(amount), Some(side), Some(symbol)) => {
                        let orderbook = self
                            .orderbooks
                            .get_mut(symbol)
                            .expect("Orderbook for symbol not found!");
                        self.last_order_id += 1;
                        msg.order_id = Some(self.last_order_id);
                        orderbook.insert_try_exec_limit(
                            &self.last_order_id,
                            side.clone(),
                            &price,
                            &amount,
                        ).to_string()
                    },
                    _ => "invalid data!".to_string(),
                }
            }
            MessageType::CancelLimitOrder => match msg.order_id {
                Some(id) => {
                    match self.orderbook_id_lookup.get_mut(&id){
                        Some(orderbook) => orderbook.cancel_limit(&id).to_string(),
                        None => "invalid id!".to_string()
                    }
                },
                _ => "no order_id given".to_string(),
            },
            MessageType::PlaceMarketOrder => "not implemented".to_string(),
        }
    }
}
