use crate::orderbook::{Orderbook};
use crate::symbol::AskOrBid;
use crate::OrderId;
use rust_decimal::Decimal;

pub struct Maker {
    order_id: OrderId,
    price: Decimal,
    filled: Decimal,
}

pub struct Match {
    pub taker: OrderId,
    pub makers: Vec<(OrderId, Decimal)>, //OrderId, filled
}

impl Match {
    //TODO: Change constructor return to Self (everywhere)
    pub fn new(taker: OrderId) -> Match {
        Match {
            taker,
            makers: Vec::new(),
        }
    }
}

pub enum MatchError {
    CantTrade,
}

pub trait OrderMatcher {
    fn match_limit(
        &self,
        orderbook: &mut Orderbook,
        order_id: &OrderId,
        side: AskOrBid,
        price: &Decimal,
        amount: &Decimal,
    ) -> Option<Match>;
    fn match_market(
        &self,
        orderbook: &mut Orderbook,
        order_id: &OrderId,
        side: AskOrBid,
        amount: &Decimal,
    ) -> Option<Match>;
}
