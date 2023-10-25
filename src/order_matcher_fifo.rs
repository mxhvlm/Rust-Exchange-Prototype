use crate::order_matcher::{Match, OrderMatcher};
use crate::orderbook::{Order, Orderbook};
use crate::symbol::AskOrBid;
use crate::OrderId;


use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;

pub struct OrderMatcherFifo {}

impl OrderMatcherFifo {
    pub fn new() -> OrderMatcherFifo {
        OrderMatcherFifo {}
    }
}

impl OrderMatcher for OrderMatcherFifo {
    fn match_limit(
        &self,
        orderbook: &mut Orderbook,
        order_id: &OrderId,
        side: AskOrBid,
        price: &Decimal,
        amount: &Decimal,
    ) -> Option<Match> {
        let (orderbook_maker, _orderbook_taker) = match side {
            AskOrBid::Ask => (&mut orderbook.orders_bid, &mut orderbook.orders_ask),
            AskOrBid::Bid => (&mut orderbook.orders_ask, &mut orderbook.orders_bid),
        };

        let mut order = Order {
            id: order_id.clone(),
            unfilled: amount.clone(),
        };
        let mut makers: Vec<(OrderId, Decimal)> = Vec::new(); //Decimal = filled
        let mut page_to_remove = None;

        // Iterate over existing pages (discrete price levels) until the order is fully matched
        'page_loop: loop {
            // Get price level of and ref to current page
            let mut pricePageTuple = match side {
                AskOrBid::Bid => orderbook_maker
                    .iter_mut()
                    .find(|(page_price, _)| *page_price <= price),
                AskOrBid::Ask => orderbook_maker
                    .iter_mut()
                    .rev()
                    .find(|(page_price, _)| *page_price >= price),
            };

            // In case a current page exists, match against the orders on it with FIFO
            if let Some((page_price, ref mut page)) = pricePageTuple {
                'order_loop: loop {
                    // Stop matching process in case our order is fully matched
                    if order.unfilled == Decimal::zero() {
                        //Order fully matched
                        break 'page_loop;
                    }

                    // Get next maker order on current page by FIFO and match against
                    // our order
                    if let Some(mut maker_entry) = page.orders.entries().next() {
                        let maker_order = maker_entry.get_mut();

                        //Maker can filly absorb the (remaining) order
                        if maker_order.unfilled > order.unfilled {
                            // Adjust the maker orders
                            maker_order.unfilled -= order.unfilled;

                            // Adjust amount of assets at current price level
                            page.amount -= order.unfilled;

                            // Add maker to the list of makers that matched our order
                            makers.push((maker_order.id, order.unfilled));

                            order.unfilled = Decimal::from(0);
                        } else { // Maker order can't fully absorb remaining order
                            order.unfilled -= maker_order.unfilled;

                            // Adjust amount of assets at current price level
                            page.amount -= maker_order.unfilled;
                            makers.push((maker_order.id, maker_order.unfilled));

                            // Remove the maker order from the book as it got completely absorbed
                            // by the taker order.
                            orderbook.orders_index.remove(&maker_order.id);

                            maker_entry.remove();
                            page_to_remove = Some(page_price.clone());
                        }
                    } else {
                        // Continue with next page in case no open orders are left
                        // on current page.
                        break 'order_loop;
                    }
                }
            } else {
                // No more pages left
                break 'page_loop;
            }

            //Delete page in case no orders are left
            if let Some(page_to_remove) = page_to_remove {
                if let Some(page) = orderbook_maker.get(&page_to_remove) {
                    //Delete page when empty
                    if page.orders.is_empty() {
                        orderbook_maker.remove(&page_to_remove);
                    }
                }
            }
        }

        if let Some(page_to_remove) = page_to_remove {
            if let Some(page) = orderbook_maker.get(&page_to_remove) {
                //Delete page when empty
                if page.orders.is_empty() {
                    orderbook_maker.remove(&page_to_remove);
                }
            }
        }

        // Insert (remaining) limit order in case our order couldn't fully be absorbed
        // by the book
        if order.unfilled > Decimal::zero() {
            orderbook._insert_limit(order.clone(), side, price.clone());
        }
        
        //Match whether any orders have been matched at all
        match order.unfilled == *amount {
            true => None,
            false => Some(Match {
                taker: order.id.clone(),
                makers,
            }),
        }
    }

    fn match_market(
        &self,
        orderbook: &mut Orderbook,
        order_id: &OrderId,
        side: AskOrBid,
        amount: &Decimal,
    ) -> Option<Match> {
        match side {
            AskOrBid::Ask => self.match_limit(orderbook, order_id, side, &Decimal::ZERO, amount),
            AskOrBid::Bid => self.match_limit(orderbook, order_id, side, &Decimal::MAX, amount),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::order_matcher::OrderMatcher;
    use crate::order_matcher_fifo::OrderMatcherFifo;
    use crate::orderbook::Orderbook;
    use crate::symbol::{AskOrBid, Symbol};
    use rust_decimal::prelude::ToPrimitive;
    use rust_decimal::Decimal;
    use rand::{thread_rng, RngCore, SeedableRng, Rng};
    use std::time::{Instant, Duration};
    use rand::distributions::{Distribution};
    use rand::rngs::ThreadRng;
    use std::convert::TryInto;
    use rand::prelude::StdRng;
    use log::info;
    use crate::symbol::AskOrBid::Ask;
    use rand_distr::Normal;


    #[test]
    fn test_match_limit_single_price_level() {
        let mut orderbook = Orderbook::new(Symbol::Asset2);
        let matcher = OrderMatcherFifo::new();

        let first_maker_id = 0;
        let second_maker_id = 1;

        let first_maker_amount = Decimal::from(32);

        let price = Decimal::from(512);

        //Write 2 ask orders at the same price
        assert_eq!(
            matcher
                .match_limit(
                    &mut orderbook,
                    &first_maker_id,
                    AskOrBid::Ask,
                    &price,
                    &first_maker_amount
                )
                .is_none(),
            true
        );
        assert_eq!(orderbook.orders_ask.get(&price).is_some(), true);
        assert_eq!(
            orderbook.orders_ask.get(&price).unwrap().amount,
            first_maker_amount
        );
        let (_id, order) = orderbook
            .orders_ask
            .get(&price)
            .unwrap()
            .orders
            .iter()
            .next()
            .unwrap();
        assert_eq!(order.unfilled, first_maker_amount);

        assert_eq!(
            matcher
                .match_limit(
                    &mut orderbook,
                    &second_maker_id,
                    AskOrBid::Ask,
                    &price,
                    &Decimal::from(16)
                )
                .is_none(),
            true
        );

        //Write a limit order that will be matched
        let mut taker_id = 2;
        let result = matcher
            .match_limit(
                &mut orderbook,
                &taker_id,
                AskOrBid::Bid,
                &price,
                &Decimal::from(31),
            )
            .unwrap();

        //Check match result
        assert_eq!(result.taker, taker_id);
        assert_eq!(result.makers.len(), 1);
        let (id, filled) = result.makers.iter().next().unwrap();
        assert_eq!(*id, first_maker_id);
        assert_eq!(*filled, Decimal::from(31));
        assert_eq!(orderbook.orders_index.get(&first_maker_id).is_some(), true); //Maker order hasn't been fully filled yet

        //Remaining amount on the page
        assert_eq!(
            orderbook.orders_ask.get(&price).unwrap().amount,
            Decimal::from(17)
        );

        //Limit that has two takers
        taker_id += 1;
        let result = matcher
            .match_limit(
                &mut orderbook,
                &taker_id,
                AskOrBid::Bid,
                &price,
                &Decimal::from(16),
            )
            .unwrap();

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
        assert_eq!(
            orderbook.orders_ask.get(&price).unwrap().amount,
            Decimal::from(1)
        );

        //Take more than remaining amount
        taker_id += 1;
        let _result = matcher
            .match_limit(
                &mut orderbook,
                &taker_id,
                AskOrBid::Bid,
                &price,
                &Decimal::from(11),
            )
            .unwrap();

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
    fn test_match_limit_multi_fill_best_price() {
        let mut orderbook = Orderbook::new(Symbol::Asset2);
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
            &first_maker_amount,
        );

        matcher.match_limit(
            &mut orderbook,
            &second_maker_id,
            AskOrBid::Ask,
            &price2,
            &first_maker_amount,
        );

        //Place limit at higher ask level, check if the lowest order gets hit
        let result = matcher
            .match_limit(
                &mut orderbook,
                &3,
                AskOrBid::Bid,
                &Decimal::ONE_THOUSAND,
                &first_maker_amount,
            )
            .unwrap();
        assert_eq!(result.makers.len(), 1);

        let mut iter = result.makers.iter();
        let (id, _filled) = iter.next().unwrap();
        assert_eq!(*id, first_maker_id);
    }

    #[test]
    fn test_match_limit_multi_insert_remaining_amount() {
        let mut orderbook = Orderbook::new(Symbol::Asset2);
        let matcher = OrderMatcherFifo::new();

        let first_maker_id = 0;
        let second_maker_id = 1;

        let first_maker_amount = Decimal::from(32);

        let price1 = Decimal::from(512);
        let price2 = Decimal::from(518);

        //Setup 2 orders at two price levels
        matcher.match_limit(
            &mut orderbook,
            &first_maker_id,
            AskOrBid::Ask,
            &price1,
            &first_maker_amount,
        );

        matcher.match_limit(
            &mut orderbook,
            &second_maker_id,
            AskOrBid::Ask,
            &price2,
            &first_maker_amount,
        );

        let price_limit = &(price2 - Decimal::ONE);
        let _result = matcher
            .match_limit(
                &mut orderbook,
                &3,
                AskOrBid::Bid,
                &price_limit,
                &Decimal::ONE_HUNDRED,
            )
            .unwrap();

        let page_written = orderbook.orders_bid.get(&price_limit);
        assert_eq!(page_written.is_some(), true);
        let page_written = page_written.unwrap();
        assert_eq!(
            page_written.amount,
            Decimal::ONE_HUNDRED - first_maker_amount
        );
        assert_eq!(page_written.orders.len(), 1);

        let order_written = page_written.orders.iter().next();
        assert_eq!(order_written.is_some(), true);

        let order_written = order_written.unwrap();
        assert_eq!(*order_written.0, order_written.1.id);
        assert_eq!(order_written.1.unfilled, page_written.amount);
    }

    #[test]
    fn test_match_limit_multi_maker_prices() {
        let mut orderbook = Orderbook::new(Symbol::Asset2);
        let matcher = OrderMatcherFifo::new();

        let mut order_id = 0;

        let first_maker_amount = Decimal::from(32);
        //Setup 2 orders at two price levels
        matcher.match_limit(
            &mut orderbook,
            &order_id,
            AskOrBid::Bid,
            &Decimal::from(4233),
            &first_maker_amount,
        );

        order_id += 1;
        matcher.match_limit(
            &mut orderbook,
            &order_id,
            AskOrBid::Bid,
            &Decimal::from(700),
            &first_maker_amount,
        );

        order_id += 1;
        matcher.match_limit(
            &mut orderbook,
            &order_id,
            AskOrBid::Bid,
            &Decimal::from(700),
            &first_maker_amount,
        );

        let price_no_touch = Decimal::from(678);
        //Insert an order that shouldn't get hit
        order_id += 1;
        matcher.match_limit(
            &mut orderbook,
            &order_id,
            AskOrBid::Bid,
            &price_no_touch,
            &first_maker_amount,
        );

        //Execute order
        order_id += 1;
        let result = matcher
            .match_limit(
                &mut orderbook,
                &order_id,
                AskOrBid::Ask,
                &Decimal::from(679),
                &(Decimal::from(4) * first_maker_amount),
            )
            .unwrap();

        //One order remaining on bid
        assert_eq!(orderbook.orders_bid.len(), 1);
        assert_eq!(orderbook.orders_bid.get(&price_no_touch).is_some(), true);
        assert_eq!(
            orderbook
                .orders_bid
                .get(&price_no_touch)
                .unwrap()
                .orders
                .iter()
                .next()
                .unwrap()
                .1
                .id,
            order_id - 1
        );

        //Matched against three makers
        assert_eq!(result.makers.len(), 3);
        for i in 0..3 {
            let (order_id, _price) = result.makers.get(i).unwrap();
            assert_eq!(*order_id, i.to_u64().unwrap());
        }
    }

    #[test]
    fn test_match_endless_loop() {
        let mut orderbook = Orderbook::new(Symbol::Asset2);
        let matcher = OrderMatcherFifo::new();

        let mut order_id = 6;
        // matcher.match_limit(&mut orderbook, &0, AskOrBid::Bid, &Decimal::from(6), &Decimal::ONE);order_id += 1;
        // matcher.match_limit(&mut orderbook, &1, AskOrBid::Ask, &Decimal::from(1), &Decimal::ONE);order_id += 1;
        matcher.match_limit(&mut orderbook, &2, AskOrBid::Bid, &Decimal::from(7), &Decimal::ONE);order_id += 1;
        matcher.match_limit(&mut orderbook, &3, AskOrBid::Ask, &Decimal::from(7), &Decimal::ONE);order_id += 1;
        matcher.match_limit(&mut orderbook, &7, AskOrBid::Ask, &Decimal::from(6), &Decimal::ONE);order_id += 1;
    }
}
