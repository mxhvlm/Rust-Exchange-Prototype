use core::fmt;
use std::collections::HashMap;
use std::fmt::Formatter;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use rust_decimal::prelude::FromStr;
use rust_decimal::Decimal;

use crate::symbol::{AskOrBid, Symbol};

/// Struct representing an async channel command of type T,
/// as well as an async channel sender that can be used to reply to the command.
/// 
/// Used by the inbound server to pass received messages onto the orderbook / matching 
/// thread.
pub struct AsyncMessage<T> {
    /// The message command
    pub cmd: T,

    /// The MPSC channel sender object that can be used to respond to the message.
    pub resp: Sender<String>,
}

/// Enum holding all the possible types of inbound exchange / order messages
/// 
/// Inbound messages will be parsed into instance of this struct.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MessageType {
    PlaceLimitOrder = 1,
    CancelLimitOrder = 2,
    PlaceMarketOrder = 3,
}

/// Struct for an inbound order message.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct InboundMessage {
    pub message_type: MessageType,
    pub symbol: Option<Symbol>,
    pub side: Option<AskOrBid>,
    pub limit_price: Option<Decimal>,
    pub amount: Option<Decimal>,
    pub order_id: Option<u64>,
}

/// Trait representing a runnable inbound server.
/// 
/// Received messages are pushed into an async channel.
pub trait InboundServer {
    /// Creates new instance of ``InboundServer`` as well as a receiver for the 
    /// async channel into which incomming messages are getting pushed
    fn new() -> (Receiver<AsyncMessage<InboundMessage>>, Self);

    /// Runs the server loop
    fn run(self);
}

impl<T> AsyncMessage<T> {
    pub fn new(msg: T) -> (AsyncMessage<T>, Receiver<String>) {
        let (resp, rx) = mpsc::channel::<String>();
        (AsyncMessage { cmd: msg, resp }, rx)
    }
}

impl MessageType {
    /// Determins whether the message type holds an amount
    fn has_amount(&self) -> bool {
        match self {
            MessageType::CancelLimitOrder => false,
            _ => true,
        }
    }

    /// Determins whether the message type holds a concrete price (ex. limit) or
    /// no price data (ex. market)
    pub fn has_price(&self) -> bool {
        match self {
            MessageType::PlaceLimitOrder => true,
            _ => false,
        }
    }

    /// Determins whether the message type holds a specific order id or not.
    /// 
    /// Cancel and lookup messages will hold an id while place orders don't 
    pub fn has_order_id(&self) -> bool {
        match self {
            MessageType::CancelLimitOrder => true,
            _ => false,
        }
    }

    /// Converts a string to a concrete MessageType.
    /// 
    /// In case the string couldn't be parsed, it'll reject the option.
    pub fn from_string(value: &String) -> Option<MessageType> {
        match value.to_lowercase().as_str() {
            "place_limit" => Some(MessageType::PlaceLimitOrder),
            "cancel_limit" => Some(MessageType::CancelLimitOrder),
            "place_market" => Some(MessageType::PlaceMarketOrder),
            _ => None,
        }
    }
}

/// Implementing 
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
        Some(value) => match T::from_str(value) {
            Ok(decimal) => Some(decimal),
            Err(_err) => None,
        },
    }
}

impl InboundMessage {
    pub fn from_hashmap(map: &HashMap<String, String>) -> Option<InboundMessage> {
        Some(InboundMessage {
            message_type: MessageType::from_string(map.get("action")?)?,
            symbol: opt_from_str_opt::<Symbol>(map.get("symbol")),
            side: opt_from_str_opt::<AskOrBid>(map.get("side")),
            limit_price: opt_from_str_opt::<Decimal>(map.get("price")),
            amount: opt_from_str_opt::<Decimal>(map.get("amount")),
            order_id: opt_from_str_opt::<u64>(map.get("order_id")),
        })
    }
}
