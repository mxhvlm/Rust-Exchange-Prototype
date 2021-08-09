use crate::order_matcher::{OrderMatcher, Match, MatchError};
use crate::orderbook::{Orderbook, Order};
use crate::OrderId;
use crate::symbol::AskOrBid;
use rust_decimal::Decimal;
use rust_decimal::prelude::Zero;

pub struct OrderMatcherFifo {

}

impl OrderMatcherFifo {
    pub fn new() -> OrderMatcherFifo {
        OrderMatcherFifo{}
    }

    /**
    Matches a market order until a certain price is reached
    */
    pub fn match_market_until(
        &self,
        orderbook: &mut Orderbook,
        order_id: &OrderId,
        side: AskOrBid,
        price: &Decimal,
        amount: &Decimal
    ) -> Option<Match> {
        let (orderbook_maker, orderbook_taker) = match side {
            AskOrBid::Ask => (&mut orderbook.orders_bid, &mut orderbook.orders_ask),
            AskOrBid::Bid => (&mut orderbook.orders_ask, &mut orderbook.orders_bid)
        };

        let mut order = Order{ id: order_id.clone(), unfilled: amount.clone() };
        let mut makers: Vec<(OrderId, Decimal)> = Vec::new(); //Decimal = filled

        loop {
            // let mut can_trade;
            // let mut best_price;

            // match side {
            //     AskOrBid::Ask => {
            //         match orderbook.get_best_bid() {
            //             None => {
            //                 can_trade = false;
            //             }
            //             Some(best_bid) => {
            //                 best_price = best_bid;
            //                 can_trade = best_bid >= *price;
            //             }
            //         }
            //
            //     },
            //     AskOrBid::Bid => {
            //         match orderbook.get_best_ask() {
            //             None => {
            //                 can_trade = false;
            //             }
            //             Some(best_bid) => {
            //                 best_price = best_bid;
            //                 can_trade = best_bid <= *price;
            //             }
            //         }
            //     }
            // }
            // if !can_trade {
            //     break;
            // }

            if let Some(page) = orderbook_maker.get_mut(&price) {
                loop {
                    let a = page.orders.entries().next();
                    if let Some(mut maker_entry) = page.orders.entries().next() {
                        let mut maker_order = maker_entry.get_mut();
                        if maker_order.unfilled > order.unfilled { //Maker can filly absorb the (remaining) order
                            maker_order.unfilled -= order.unfilled;
                            page.amount -= order.unfilled;
                            makers.push((maker_order.id, order.unfilled));

                            order.unfilled = Decimal::from(0);
                        } else {
                            order.unfilled -= maker_order.unfilled;
                            page.amount -= maker_order.unfilled;
                            makers.push((maker_order.id, maker_order.unfilled));

                            orderbook.orders_index.remove(&maker_order.id);

                            maker_entry.remove();
                        }
                    } else { //No more orders left on page
                        break;
                    }
                    if order.unfilled == Decimal::from(0) { //Order fully matched
                        break;
                    }
                }
            }
        }

        if let Some(page) = orderbook_maker.get(&price) {
            if page.orders.is_empty() {
                orderbook_maker.remove(&price);
            }
        }

        if order.unfilled > Decimal::zero() {
            orderbook._insert_limit(order.clone(), side, price.clone());
        }

        match order.unfilled == *amount { //Match whether any orders have been matched at all
            true => None,
            false => {
                Some(Match {
                    taker: order.id.clone(),
                    makers
                })
            }
        }
    }
}

impl OrderMatcher for OrderMatcherFifo {
    //TODO: Limit fill best price not the price specified
    fn match_limit(&self, orderbook: &mut Orderbook, order_id: &OrderId, side: AskOrBid, price: &Decimal, amount: &Decimal) -> Option<Match> {
        let (orderbook_maker, orderbook_taker) = match side {
            AskOrBid::Ask => (&mut orderbook.orders_bid, &mut orderbook.orders_ask),
            AskOrBid::Bid => (&mut orderbook.orders_ask, &mut orderbook.orders_bid)
        };

        let mut order = Order{ id: order_id.clone(), unfilled: amount.clone() };
        let mut makers: Vec<(OrderId, Decimal)> = Vec::new(); //Decimal = filled

        if let Some(page) = orderbook_maker.get_mut(price) {
            loop {
                let a = page.orders.entries().next();
                if let Some(mut maker_entry) = page.orders.entries().next() {
                    let mut maker_order = maker_entry.get_mut();
                    if maker_order.unfilled > order.unfilled { //Maker can filly absorb the (remaining) order
                        maker_order.unfilled -= order.unfilled;
                        page.amount -= order.unfilled;
                        makers.push((maker_order.id, order.unfilled));

                        order.unfilled = Decimal::from(0);
                    } else {
                        order.unfilled -= maker_order.unfilled;
                        page.amount -= maker_order.unfilled;
                        makers.push((maker_order.id, maker_order.unfilled));

                        orderbook.orders_index.remove(&maker_order.id);

                        maker_entry.remove();
                    }
                } else { //No more orders left on page
                    break;
                }
                if order.unfilled == Decimal::from(0) { //Order fully matched
                    break;
                }
            }
        }
        if let Some(page) = orderbook_maker.get(&price) {
            if page.orders.is_empty() {
                orderbook_maker.remove(&price);
            }
        }

        if order.unfilled > Decimal::zero() {
            orderbook._insert_limit(order.clone(), side, price.clone());
        }

        match order.unfilled == *amount { //Match whether any orders have been matched at all
            true => None,
            false => {
                Some(Match {
                    taker: order.id.clone(),
                    makers
                })
            }
        }
    }

    fn match_market(&self, orderbook: &mut Orderbook, order: &Order) -> Option<Match> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::orderbook::Orderbook;
    use crate::symbol::{Symbol, AskOrBid};
    use crate::order_matcher_fifo::OrderMatcherFifo;
    use crate::order_matcher::OrderMatcher;
    use rust_decimal::Decimal;

    #[test]
    fn test_match_limit_single_price_level() {
        let mut orderbook = Orderbook::new(Symbol::ETH);
        let matcher = OrderMatcherFifo::new();

        let first_maker_id = 0;
        let second_maker_id = 1;

        let first_maker_amount = Decimal::from(32);

        let price = Decimal::from(512);

        //Write 2 ask orders at the same price
        assert_eq!(matcher.match_limit(
            &mut orderbook,
            &first_maker_id,
            AskOrBid::Ask,
            &price,
            &first_maker_amount
        ).is_none(), true);
        assert_eq!(orderbook.orders_ask.get(&price).is_some(), true);
        assert_eq!(orderbook.orders_ask.get(&price).unwrap().amount, first_maker_amount);
        let (id, order) = orderbook.orders_ask.get(&price).unwrap().orders.iter().next().unwrap();
        assert_eq!(order.unfilled, first_maker_amount);

        assert_eq!(matcher.match_limit(
            &mut orderbook,
            &second_maker_id,
            AskOrBid::Ask,
            &price,
                &Decimal::from( 16)
        ).is_none(), true);

        //Write a limit order that will be matched
        let mut taker_id = 2;
        let result = matcher.match_limit(
            &mut orderbook,
            &taker_id,
            AskOrBid::Bid,
            &price,
            &Decimal::from(31)
        ).unwrap();

        //Check match result
        assert_eq!(result.taker, taker_id);
        assert_eq!(result.makers.len(), 1);
        let (id, filled) = result.makers.iter().next().unwrap();
        assert_eq!(*id, first_maker_id);
        assert_eq!(*filled, Decimal::from(31));
        assert_eq!(orderbook.orders_index.get(&first_maker_id).is_some(), true); //Maker order hasn't been fully filled yet

        //Remaining amount on the page
        assert_eq!(orderbook.orders_ask.get(&price).unwrap().amount, Decimal::from(17));

        //Limit that has two takers
        taker_id += 1;
        let result = matcher.match_limit(
            &mut orderbook,
            &taker_id,
            AskOrBid::Bid,
            &price,
            &Decimal::from(16)
        ).unwrap();

        assert_eq!(result.taker, taker_id);
        assert_eq!(result.makers.len(), 2);
        let mut iter = result.makers.iter();
        let (id, filled) = iter.next().unwrap();
        assert_eq!(*id, first_maker_id);
        assert_eq!(*filled, Decimal::from(1));
        let (id, filled) = iter.next().unwrap();
        assert_eq!(*id, second_maker_id);
        assert_eq!(*filled, Decimal::from(15));
        //First maker has been fully filled
        assert_eq!(orderbook.orders_index.get(&first_maker_id).is_some(), false);

        //Remaining amount on the page
        assert_eq!(orderbook.orders_ask.get(&price).unwrap().amount, Decimal::from(1));

        //Take more than remaining amount
        taker_id += 1;
        let result = matcher.match_limit(
            &mut orderbook,
            &taker_id,
            AskOrBid::Bid,
            &price,
            &Decimal::from(11)
        ).unwrap();

        //Page gets removed after last order is filled
        assert_eq!(orderbook.orders_ask.get(&price).is_none(), true);

        //Check whether remaining order was written on the orderbook of taker
        let new_page = orderbook.orders_bid.get(&price).unwrap();
        assert_eq!(new_page.amount, Decimal::from(10));
        assert_eq!(new_page.orders.len(), 1);

        let remaining_order = new_page.orders.iter().next().unwrap().1;
        assert_eq!(remaining_order.id, taker_id);
        assert_eq!(remaining_order.unfilled, Decimal::from(10));
    }

    #[test]
    fn test_match_limit_multi_price_level() {
        let mut orderbook = Orderbook::new(Symbol::ETH);
        let matcher = OrderMatcherFifo::new();

        let first_maker_id = 0;
        let second_maker_id = 1;

        let first_maker_amount = Decimal::from(32);

        let price1 = Decimal::from(512);
        let price2 = Decimal::from(513);

        //Setup 2 orders at two price levels
        matcher.match_limit(
            &mut orderbook,
            &first_maker_id,
            AskOrBid::Ask,
            &price1,
            &first_maker_amount
        );

        matcher.match_limit(
            &mut orderbook,
            &second_maker_id,
            AskOrBid::Ask,
            &price2,
            &first_maker_amount
        );

        //Place limit at higher ask level, check if the
    }
}