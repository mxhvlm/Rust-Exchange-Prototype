//TODO Read Symbols from file

use std::fmt;

#[derive(Debug)]
pub enum Symbol {
    BTC = 1,
    ETH = 2,
}

#[derive(Debug)]
pub enum AskOrBuy {
    Ask = 0,
    Buy = 1
}

impl AskOrBuy {
    pub fn from_u8(value: u8) -> Option<AskOrBuy> {
        match value {
            0 => Some(AskOrBuy::Ask),
            1 => Some(AskOrBuy::Buy),
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