use std::sync::mpsc::{Receiver, Sender};
use std::io::ErrorKind;
use std::sync::mpsc;
use crate::symbol::{Symbol, AskOrBid};
use rust_decimal::Decimal;
use core::fmt;
use std::fmt::Formatter;
use std::collections::HashMap;
use rust_decimal::prelude::FromStr;

pub trait InboundServer {
    fn new() -> (Receiver<AsyncMessage<InboundMessage>>, Self);
    fn run(self);
}

pub struct AsyncMessage<T> {
    pub cmd: T,
    pub resp: Sender<Result<String, ErrorKind>>, //TODO: implement custom error type
}

impl<T> AsyncMessage<T> {
    pub fn new(msg: T) -> (AsyncMessage<T>, Receiver<Result<String, ErrorKind>>) {
        let (resp, rx) = mpsc::channel::<Result<String, ErrorKind>>();
        (AsyncMessage{ cmd: msg, resp}, rx)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct InboundMessage {
    pub message_type: MessageType,
    pub symbol: Symbol,
    pub side: AskOrBid,
    pub limit_price: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub order_id: Option<u64>
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MessageType {
    PlaceLimitOrder = 1,
    DeleteLimitOrder = 2,
    PlaceMarketOrder = 3
}

impl MessageType {
    pub(crate) fn has_amount(&self) -> bool {
        match self {
            MessageType::DeleteLimitOrder => false,
            _ => true
        }
    }

    pub fn has_price(&self) -> bool {
        match self {
            MessageType::PlaceLimitOrder => true,
            _ => false
        }
    }

    pub fn has_order_id(&self) -> bool {
        match self {
            MessageType::DeleteLimitOrder => true,
            _ => false
        }
    }

    pub fn from_string(value: &String) -> Option<MessageType> {
        match value.to_lowercase().as_str() {
            "place_limit" => Some(MessageType::PlaceLimitOrder),
            "remove_limit" => Some(MessageType::DeleteLimitOrder),
            "place_market" => Some(MessageType::PlaceMarketOrder),
            _ => None
        }
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/**
Trys to convert a given Option<&String> into a generic given that it implements the FromStr trait.
Returns Some(result) if and only if the conversion is possible and the input was Some(value)
Returns None in all other cases

Used for parsing values that might be in the HashMap to Option<T> when creating an InboundMessage
from a HashMap
*/
fn opt_from_str_opt<T: FromStr>(value: Option<&String>) -> Option<T> {
    match value {
        None => None,
        Some(value) => {
            match T::from_str(value) {
                Ok(decimal) => Some(decimal),
                Err(err) => None
            }
        }
    }
}

impl InboundMessage {
    pub fn get_dummy() -> InboundMessage {
        InboundMessage {
            message_type: MessageType::PlaceLimitOrder,
            symbol: Symbol::BTC,
            side: AskOrBid::Ask,
            limit_price: Some(Decimal::from(512)),
            amount: Some(Decimal::from(20)),
            order_id: Some(182349)
        }
    }

    pub fn has_amount(&self) -> bool{
        return self.amount.is_some();
    }

    pub fn has_order_id(&self) -> bool{
        return self.order_id.is_some();
    }

    pub fn has_price(&self) -> bool{
        return self.limit_price.is_some();
    }

    //TODO: Research whether HashMap ist the right datatype for small numbers of key,value pairs
    pub fn from_hashmap(map: &HashMap<String, String>) -> Option<InboundMessage> {
        Some(InboundMessage {
            message_type: MessageType::from_string(map.get("action")?)?,
            symbol: Symbol::from_string(map.get("symbol")?)?,
            side: AskOrBid::from_string(map.get("side")?)?,
            limit_price: opt_from_str_opt::<Decimal>(map.get("price")),
            amount: opt_from_str_opt::<Decimal>(map.get("amount")),
            order_id:opt_from_str_opt::<u64>(map.get("order_id"))
        })
    }
}