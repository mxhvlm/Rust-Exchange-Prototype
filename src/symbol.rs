//TODO Read Symbols from file

use std::fmt;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Symbol {
    BTC = 1,
    ETH = 2,
}

#[derive(Debug)]
pub enum AskOrBid {
    Ask = 0,
    Bid = 1
}

impl AskOrBid {
    pub fn from_u8(value: u8) -> Option<AskOrBid> {
        match value {
            0 => Some(AskOrBid::Ask),
            1 => Some(AskOrBid::Bid),
            _ => None
        }
    }
}

impl Symbol {
    pub fn from_u8(value: u8) -> Option<Symbol> {
        match value {
            1 => Some(Symbol::BTC),
            2 => Some(Symbol::ETH),
            _ => None
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}