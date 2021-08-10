//TODO Read Symbols from file

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum Symbol {
    BTC = 1,
    ETH = 2,
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum AskOrBid {
    Ask = 0,
    Bid = 1,
}

impl FromStr for AskOrBid {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ask" => Ok(AskOrBid::Ask),
            "bid" => Ok(AskOrBid::Bid),
            _ => Err(()),
        }
    }
}

impl FromStr for Symbol {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "btc" => Ok(Symbol::BTC),
            "eth" => Ok(Symbol::ETH),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
