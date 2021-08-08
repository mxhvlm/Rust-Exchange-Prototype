
use std::collections::HashMap;
use std::io::ErrorKind;


use log::{error, info};

use crate::inbound_http_server::InboundHttpServer;
use crate::inbound_server::{InboundMessage, InboundServer, MessageType};
use crate::orderbook::{Orderbook, InsertLimitResult};
use crate::symbol::Symbol;
use crate::OrderId;
use json::JsonValue;


pub struct ExchangeCore {
    orderbooks: HashMap<Symbol, Orderbook>,
    orderbook_id_lookup: HashMap<OrderId, Symbol>,
    last_order_id: OrderId,
}

impl ExchangeCore {
    pub fn new() -> ExchangeCore {
        let mut orderbooks = HashMap::new();
        let mut orderbook_id_lookup = HashMap::new();

        orderbooks.insert(Symbol::BTC, Orderbook::new(Symbol::BTC));
        orderbooks.insert(Symbol::ETH, Orderbook::new(Symbol::ETH));

        ExchangeCore {
            orderbooks,
            last_order_id: 0,
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

    //TODO: When implementing multithreading, we need to be able to route orderflow based on symbol as quickly as possible
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

                        let result = orderbook.insert_try_exec_limit(
                            &self.last_order_id,
                            side.clone(),
                            &price,
                            &amount,
                        );

                        if result.is_success() {
                            self.orderbook_id_lookup.insert(self.last_order_id, symbol.clone());
                        }

                        JsonValue::from(result).to_string()
                    },
                    _ => "invalid data!".to_string(),
                }
            }
            MessageType::CancelLimitOrder => match msg.order_id {
                Some(id) => {
                    match self.orderbook_id_lookup.get(&id) {
                        Some(symbol) => {
                            self.orderbooks.get_mut(symbol).unwrap().cancel_limit(&id).to_string()
                        },
                        None => "invalid id!".to_string()
                    }
                },
                _ => "no order_id given".to_string(),
            },
            MessageType::PlaceMarketOrder => "not implemented".to_string(),
        }
    }
}
