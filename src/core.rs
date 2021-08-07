use log::{info, error};
use std::collections::HashMap;
use crate::inbound_server::{InboundServer, InboundMessage, MessageType};
use crate::orderbook::Orderbook;
use crate::symbol::Symbol;
use crate::inbound_http_server::InboundHttpServer;
use std::io::ErrorKind;
use core::num::FpCategory::Infinite;
use json::object;

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
        let (inbound_reciever, inbound_server) = InboundHttpServer::new();

        inbound_server.run();

        loop {
            if let Ok(msg) = inbound_reciever.try_recv() {
                let mut cmd = msg.cmd.clone();
                info!("Processing inbound message: {:?}...", &cmd);
                msg.resp.send(match self.process_inbound_message(&mut cmd) {
                    true => Ok(format!("{}", cmd.order_id.unwrap())),
                    false => {
                        error!("Invalid limit order: {:?}", &cmd);
                        Err(ErrorKind::InvalidData)
                    }
                }).unwrap();
            }
        }
    }

    //TODO: find some other way of returning the msg.order_id
    //TODO: handle the way msg.symbol gets checked properly
    fn process_inbound_message(&mut self, msg: &mut InboundMessage) -> bool {
        match &msg.symbol {
            Some(symbol) => {
                let mut orderbook = self.orderbooks.get_mut(symbol).expect("Orderbook for symbol not found!");
                match msg.message_type {
                    MessageType::PlaceLimitOrder => {
                        match (msg.limit_price, msg.amount, msg.side.clone()) {
                            (Some(price), Some(amount), Some(side)) => {
                                self.last_order_id += 1;
                                msg.order_id = Some(self.last_order_id);
                                orderbook.insert_try_exec_limit(&self.last_order_id, side, &price, &amount)
                            }
                            _ => false
                        }

                    }
                    MessageType::DeleteLimitOrder => {
                        match msg.order_id {
                            Some(id) => orderbook.remove_limit(&id),
                            _ => false
                        }

                    }
                    MessageType::PlaceMarketOrder => {
                        false
                    }
                }
            },
            None => false
        }
    }
}