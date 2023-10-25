/// Main module for the exchange prototype

use std::collections::HashMap;

use log::{info};

use crate::inbound_http_server::InboundHttpServer;
use crate::inbound_server::{InboundMessage, InboundServer, MessageType};
use crate::orderbook::{CancelLimitResult, Orderbook};
use crate::symbol::Symbol;
use crate::OrderId;
use json::JsonValue;

/// Struct holding all all exchange data
/// 
/// 
pub struct ExchangeCore {
    /// Mapping between symbol and orderbook
    /// Used by the handler for incoming messages to look up the correct
    /// orderbook for a given symbol
    orderbooks: HashMap<Symbol, Orderbook>,

    /// Mapping between OrderId and Symbol, used for lookup and cancel messages
    orderbook_id_lookup: HashMap<OrderId, Symbol>,

    /// Global seq number for orders, shared accross books
    last_order_id: OrderId,
}

impl ExchangeCore {
    pub fn new() -> ExchangeCore {
        let mut orderbooks = HashMap::new();
        let orderbook_id_lookup = HashMap::new();

        orderbooks.insert(Symbol::Asset1, Orderbook::new(Symbol::Asset1));
        orderbooks.insert(Symbol::Asset2, Orderbook::new(Symbol::Asset2));

        ExchangeCore {
            orderbooks,
            last_order_id: 0,
            orderbook_id_lookup,
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

    // Processes an ``InboundMessage`` by resolving the order book and inserting the order
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

                        let limit_result = orderbook.insert_try_exec_limit(
                            &self.last_order_id,
                            side.clone(),
                            &price,
                            &amount,
                        );

                        if limit_result.is_success() {
                            self.orderbook_id_lookup
                                .insert(self.last_order_id, symbol.clone());
                        }

                        JsonValue::from(limit_result).to_string()
                    }
                    _ => "invalid data!".to_string(),
                }
            }

            MessageType::CancelLimitOrder => match msg.order_id {
                Some(id) => match self.orderbook_id_lookup.get_mut(&id) {
                    Some(symbol) => {
                        let limit_result = self.orderbooks.get_mut(symbol).unwrap().cancel_limit(&id);
                        if let CancelLimitResult::Success = limit_result {
                            self.orderbook_id_lookup.remove(&id);
                        }
                        limit_result.to_string()
                    }
                    None => "invalid id!".to_string(),
                },
                _ => "no order_id given".to_string(),
            },
            MessageType::PlaceMarketOrder => "not implemented".to_string(), //TODO: Use existing implementation
        }
    }
}
