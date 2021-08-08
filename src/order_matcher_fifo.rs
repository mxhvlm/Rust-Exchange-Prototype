use crate::order_matcher::{OrderMatcher, Match, MatchError};
use crate::orderbook::{Orderbook, Order};
use crate::OrderId;

pub struct OrderMatcherFifo;

impl OrderMatcher for OrderMatcherFifo {
    fn match_limit(orderbook: &mut Orderbook, order_id: &OrderId) -> Result<Match, MatchError> {
        if !orderbook.can_match() {
            return Err(MatchError::CantTrade);
        }

        if let Some(order) = orderbook.get_order_mut(order_id) {
            //let side = orderbook.get_side_for_price()
        }

        //The side with the maker has only one order in it
        // if orderbook.orders_ask.get_mut(
        //     orderbook.get_best_ask().unwrap()).unwrap().orders.len() == 1 {
        //
        // }

        //let order_match = Match::new
        Err(MatchError::CantTrade)

    }

    fn match_market(orderbook: &mut Orderbook, order: &Order) {
        todo!()
    }
}