use std::fmt::Formatter;
use std::fmt;
use crate::symbol::{Symbol, AskOrBid};
use rust_decimal::Decimal;
use std::io::ErrorKind;
use rand::Rng;
use log::info;

#[derive(Debug)]
pub struct InboundMessage {
    pub message_type: MessageType,
    pub symbol: Symbol,
    pub side: AskOrBid,
    pub limit_price: Decimal,
    pub amount: Decimal
}

#[derive(Debug, Clone)]
pub enum MessageType {
    PLACE_LIMIT_ORDER = 1,
    DELETE_LIMIT_ORDER = 2,
    PLACE_MARKET_ORDER = 3
}

impl MessageType {
    pub(crate) fn from_u8(value: u8) -> Option<MessageType> {
        match value {
            1 => Some(MessageType::PLACE_LIMIT_ORDER),
            2 => Some(MessageType::DELETE_LIMIT_ORDER),
            3 => Some(MessageType::PLACE_MARKET_ORDER),
            _ => None
        }
    }

    pub(crate) fn has_volume(&self) -> bool {
        match self {
            MessageType::DELETE_LIMIT_ORDER => false,
            _ => true
        }
    }

    pub fn has_price(&self) -> bool {
        match self {
            MessageType::PLACE_LIMIT_ORDER => true,
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
    pub fn from_bytes(buff: Vec<u8>) -> Result<InboundMessage, ErrorKind> {
        info!("{}", buff.len());
        match buff.len() {
            MSG_SIZE => {
                let mut iter = buff.into_iter();

                let symbol = Symbol::from_u8(iter.next().unwrap())
                    .expect("invlaid symbol");

                let message_type = MessageType::from_u8(iter.next().unwrap())
                    .expect("invalid message num");

                let side = AskOrBid::from_u8(iter.next().unwrap())
                    .expect("invalid AskOrBuy");

                let limit_price = match message_type { //TODO: Properly read Decimals
                    MessageType::PLACE_MARKET_ORDER => Decimal::from(0),
                    _ => Decimal::from(500 + rand::thread_rng().gen_range(0..100)),
                };

                Ok(InboundMessage{
                    symbol,
                    side,
                    message_type: message_type.clone(),
                    limit_price,
                    amount:
                    if message_type.has_volume()
                    {
                        Decimal::from(rand::thread_rng().gen_range(50..100))
                    } else {Decimal::from(-1)}
                })
            },
            _ => Err(ErrorKind::InvalidData)
        }
    }
}