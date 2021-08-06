use std::sync::mpsc::{Receiver, Sender};
use std::io::ErrorKind;
use std::sync::mpsc;
use crate::symbol::{Symbol, AskOrBid};
use rust_decimal::Decimal;
use core::fmt;
use std::fmt::Formatter;

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
    pub limit_price: Decimal,
    pub amount: Decimal
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MessageType {
    PlaceLimitOrder = 1,
    DeleteLimitOrder = 2,
    PlaceMarketOrder = 3
}

impl MessageType {
    pub(crate) fn has_volume(&self) -> bool {
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
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl InboundMessage {
    pub fn get_dummy() -> InboundMessage {
        InboundMessage {
            message_type: MessageType::PlaceLimitOrder,
            symbol: Symbol::BTC,
            side: AskOrBid::Ask,
            limit_price: Decimal::from(512),
            amount: Decimal::from(20)
        }
    }
}