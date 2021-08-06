//TODO Read Symbols from file

use std::fmt;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Symbol {
    BTC = 1,
    ETH = 2,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AskOrBid {
    Ask = 0,
    Bid = 1
}

impl AskOrBid {
    pub fn from_string(value: &String) -> Option<AskOrBid> {
        match value.to_lowercase().as_str() {
            "ask" => Some(AskOrBid::Ask),
            "bid" => Some(AskOrBid::Bid),
            _ => None
        }
    }
}

impl Symbol {
    pub fn from_string(value: &String) -> Option<Symbol> {
        match value.to_lowercase().as_str() {
            "btc" => Some(Symbol::BTC),
            "eth" => Some(Symbol::ETH),
            _ => None
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}