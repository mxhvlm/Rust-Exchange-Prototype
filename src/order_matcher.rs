use crate::orderbook::{Orderbook, Order};
use crate::OrderId;
use rust_decimal::Decimal;

pub struct Maker {
    order_id: OrderId,
    price: Decimal,
    filled: Decimal
}

pub struct Match {
    taker: OrderId,
    makers: Vec<Maker>
}

impl Match {
    //TODO: Change constructor return to Self (everywhere)
    pub fn new(taker: OrderId) -> Match {
        Match {
            taker,
            makers: Vec::new()
        }
    }
}

pub enum MatchError {
    CantTrade
}

pub trait OrderMatcher {
    fn match_limit(orderbook: &mut Orderbook, order_id: &OrderId) -> Result<Match, MatchError>;
    fn match_market(orderbook: &mut Orderbook, order: &Order);
}
