use std::collections::{BTreeMap, HashMap};

use log::info;
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use json::{object, JsonValue};

use crate::symbol::{AskOrBid, Symbol};
use crate::OrderId;

use core::fmt;
use crate::order_matcher_fifo::OrderMatcherFifo;

//TODO: Handle Decimal scales


//TODO: Implement InsertLimitResult as Result<>?
#[derive(PartialEq, Debug)]
pub enum InsertLimitResult {
    Success(OrderId),
    PartiallyFilled(OrderId, Decimal),
    FullyFilled,
    OrderDataInvalid
}

#[derive(PartialEq, Debug)]
pub enum CancelLimitResult {
    Success,
    OrderIdNotFound
}

pub struct OrderbookPage {
    pub orders: HashMap<OrderId, Order>,
    pub amount: Decimal,
}

pub struct Orderbook {
    symbol: Symbol,
    pub orders_ask: BTreeMap<Decimal, OrderbookPage>,
    pub orders_bid: BTreeMap<Decimal, OrderbookPage>,
    pub orders_index: HashMap<OrderId, Decimal>, //TODO: Find a way to always guarantee that orders and index are consistent
    order_matcher: OrderMatcherFifo
}

#[derive(Clone, PartialEq, Debug)]
pub struct Order {
    pub id: OrderId,
    pub unfilled: Decimal,
}

impl InsertLimitResult {
    pub fn is_success(&self) -> bool {
        match self {
            InsertLimitResult::Success(_) => true,
            InsertLimitResult::PartiallyFilled(_, _) => true,
            InsertLimitResult::FullyFilled => true,
            InsertLimitResult::OrderDataInvalid => false,
        }
    }
}

impl fmt::Display for InsertLimitResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InsertLimitResult::Success(_) => write!(f, "success"),
            InsertLimitResult::PartiallyFilled(_, _) => write!(f, "partially_filled"),
            InsertLimitResult::FullyFilled => write!(f, "fully_filled"),
            InsertLimitResult::OrderDataInvalid => write!(f, "order_data_invalid"),
        }
    }
}

impl From<InsertLimitResult> for JsonValue {
    fn from(result: InsertLimitResult) -> Self {
        let status = result.to_string();
        match result {
            InsertLimitResult::Success(order_id) => {
                object!{
                    "status" => status,
                    "order_id" => order_id
                }
            },
            InsertLimitResult::OrderDataInvalid => {
                object! {
                    "status" => status
                }
            },
            InsertLimitResult::PartiallyFilled(order_id, unfilled) => {
                object! {
                    "status" => status,
                    "order_id" => order_id,
                    "remaining" => unfilled.to_string()
                }
            },
            InsertLimitResult::FullyFilled => {
                object!{
                    "status" => status
                }
            }
        }
    }
}

impl fmt::Display for CancelLimitResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl OrderbookPage {
    fn new(order: &Order) -> OrderbookPage {
        let mut orders = HashMap::<OrderId, Order>::new();
        let amount = order.unfilled;
        orders.insert(order.id, order.clone());
        OrderbookPage { orders, amount }
    }

    fn remove(&mut self, order_id: &OrderId) -> Option<Order> {
        if let Some(removed) = self.orders.remove(order_id) {
            self.amount -= removed.unfilled;
            return Some(removed);
        }
        None
    }

    fn get(&self, order_id: &OrderId) -> Option<&Order> {
        self.orders.get(order_id)
    }

    fn insert(&mut self, order: &Order) {
        self.orders.insert(order.id, order.clone());
        self.amount += order.unfilled;
    }
}

impl Orderbook {
    pub fn new(symbol: Symbol) -> Orderbook {
        Orderbook {
            symbol,
            orders_ask: BTreeMap::<Decimal, OrderbookPage>::new(),
            orders_bid: BTreeMap::<Decimal, OrderbookPage>::new(),
            orders_index: HashMap::<OrderId, Decimal>::new(),
            order_matcher: OrderMatcherFifo {},
        }
    }

    pub fn get_best_ask(&self) -> Option<Decimal> {
        self.orders_ask.iter().next().map(|(price, _)| *price)
    }

    pub fn get_best_bid(&self) -> Option<Decimal> {
        self.orders_bid.iter().rev().next().map(|(price, _)| *price)
    }

    pub fn contains_order(&self, order_id: &OrderId) -> bool {
        self.orders_index.contains_key(order_id)
    }

    /**
    Returns None if either no orders are in the orderbook or if the orderbook is in an inconsistent state
     */
    pub fn get_side_for_price(
        &self,
        price: &Decimal,
    ) -> Option<AskOrBid> {
        //Orderbook is in an inconsistent state eg. get_best_buy() >= get_best_bid()
        if self.can_match() {
            return None;
        }
        if let Some(best_ask) = self.get_best_ask() {
            if *price >= best_ask {
                return Some(AskOrBid::Ask);
            }
        }
        if let Some(best_bid) = self.get_best_bid() {
            if *price <= best_bid {
                return Some(AskOrBid::Bid);
            }
        }
        None
    }

    pub fn get_order_mut(&mut self, order_id: &OrderId) -> Option<&mut Order> {
        if let Some(price) = self.orders_index.get(order_id) {
            if let Some(side) = self.get_side_for_price(&price) {
                let orderbook = match side {
                    AskOrBid::Ask => &mut self.orders_ask,
                    AskOrBid::Bid => &mut self.orders_bid,
                };

                if let Some(order_page) = orderbook.get_mut(&price) {
                    return order_page.orders.get_mut(order_id);
                }
            }
        }
        return None;
    }

    /*pub fn get_unfilled(&self, order_id: &OrderId) -> Option<Decimal> {
        match self.contains_order(order_id) {
            true => {
                let page = self.orders_index.
            },
            false => None
        }
    }*/

    pub fn can_match(&self) -> bool {
        let best_ask = self.get_best_ask();
        let best_bid = self.get_best_bid();

        if let Some(best_ask) = best_ask {
            if let Some(best_bid) = best_bid {
                return best_bid >= best_ask;
            }
        }
        false
    }

    fn log_best_ask_bid(&self) {
        info!(
            "Best Ask: {}",
            self.get_best_ask().unwrap_or(Decimal::from(-1))
        );
        info!(
            "Best Bid: {}",
            self.get_best_bid().unwrap_or(Decimal::from(-1))
        );
    }

    pub fn insert_try_exec_limit(
        &mut self,
        order_id: &OrderId,
        side: AskOrBid,
        price: &Decimal,
        size: &Decimal,
    ) -> InsertLimitResult {
        let order_id = order_id.clone();
        let size = size.clone();
        let price = price.clone();

        if let InsertLimitResult::OrderDataInvalid = self.insert_limit(order_id, side.clone(), price, size) {
            return InsertLimitResult::OrderDataInvalid;
        }

        match self.can_match() {
            true => {

                InsertLimitResult::PartiallyFilled(order_id, Decimal::default())
            },
            false => InsertLimitResult::Success(order_id)
        }
    }

    fn insert_limit(
        &mut self,
        order_id: OrderId,
        side: AskOrBid,
        price: Decimal,
        size: Decimal,
    ) -> InsertLimitResult {
        if price <= Decimal::zero() || size <= Decimal::zero() {
            return InsertLimitResult::OrderDataInvalid;
        }

        //Return false if an order with the same id is already inserted into orderbook
        if self.orders_index.contains_key(&order_id) {
            return InsertLimitResult::OrderDataInvalid;
        }

        let order = Order {
            id: order_id,
            unfilled: size,
        };

        let orderbook = match side {
            AskOrBid::Ask => &mut self.orders_ask,
            AskOrBid::Bid => &mut self.orders_bid,
        };

        orderbook
            .entry(price)
            .and_modify(|page| {
                page.insert(&order)
            })
            .or_insert_with(|| OrderbookPage::new(&order));

        self.orders_index.insert(order_id, price);

        info!("Inserted order {} at price {}", order_id, price);
        self.log_best_ask_bid();

        InsertLimitResult::Success(order_id)
    }

    pub fn cancel_limit(&mut self, order_id: &OrderId) -> CancelLimitResult {
        if let Some(price) = self.orders_index.get(order_id) {
            if let Some(side) = self.get_side_for_price(price) {
                let orderbook = match side {
                    AskOrBid::Ask => &mut self.orders_ask,
                    AskOrBid::Bid => &mut self.orders_bid,
                };
                if let Some(orderbook_page) = orderbook.get_mut(price) {
                    if let Some(removed) = orderbook_page.remove(order_id) {
                        if orderbook_page.amount == Decimal::from(0) {
                            orderbook.remove(price);
                        }
                        return CancelLimitResult::Success
                    }
                }
            } else {
                panic!("cancel_limit called in an inconsistent state!")
            }
        }
        CancelLimitResult::OrderIdNotFound
    }
}

#[cfg(test)]
mod orderbook_tests {
    use std::hash::Hash;

    use rand::{RngCore, SeedableRng};

    use super::*;

    //Wrapper for isolated testing of Orderbook::insert_limit()
    fn insert_limit(
        orderbook: &mut Orderbook,
        order_id: &OrderId,
        side: AskOrBid,
        price: &Decimal,
        size: &Decimal,
    ) -> InsertLimitResult {
        let order_id = order_id.clone();
        let size = size.clone();
        let price = price.clone();

        orderbook.insert_limit(order_id, side, price, size)
    }

    #[test]
    fn test_new_page() {
        let order = Order {
            id: 0,
            unfilled: Decimal::from(10),
        };
        let page = OrderbookPage::new(&order);

        assert_eq!(page.orders.len(), 1);
        assert_eq!(
            order,
            page.orders
                .iter()
                .next()
                .map(|(_, order)| order.clone())
                .unwrap()
        );

        assert_eq!(order.unfilled, page.amount);
    }

    #[test]
    fn test_page_remove_order() {
        let order1_amount = Decimal::from(10);
        let order2_amount = Decimal::from(3244);

        let order = Order {
            id: 0,
            unfilled: order1_amount,
        };
        let mut page = OrderbookPage::new(&order);
        page.insert(&Order{ id: 1, unfilled: order2_amount});

        let removed_order = page.remove(&0);
        assert_eq!(removed_order, Some(order));
        assert_eq!(page.amount, order2_amount);
        assert_eq!(page.remove(&0), None);
    }

    #[test]
    fn test_orderbook_insert_limit() {
        let mut orderbook = Orderbook::new(Symbol::BTC);
        let mut id = 16u64;
        let price = Decimal::from(100);
        let unfilled = Decimal::from(16);

        //Adding limit order with an unused order_id
        assert_eq!(
            insert_limit(&mut orderbook, &id, AskOrBid::Ask, &price, &unfilled),
            InsertLimitResult::Success(id)
        );

        //Check if order got written into the BTree
        assert_eq!(
            orderbook
                .orders_ask
                .get(&price)
                .unwrap()
                .orders
                .get(&id)
                .unwrap()
                .id,
            id
        );

        //Check if index got written into HashMap
        assert_eq!(orderbook.orders_index.get(&id).unwrap(), &price);

        //Adding an order with the same order_id twice shouldn't be possible.
        assert_eq!(
            insert_limit(&mut orderbook, &id, AskOrBid::Ask, &price, &unfilled),
            InsertLimitResult::OrderDataInvalid
        );

        //Check bid side (Don't hanve to check indicies since there is only one HashMap
        id += 1;
        assert_eq!(
            insert_limit(&mut orderbook, &id, AskOrBid::Bid, &price, &unfilled),
            InsertLimitResult::Success(id)
        );
        assert_eq!(
            orderbook
                .orders_bid
                .get(&price)
                .unwrap()
                .orders
                .get(&id)
                .unwrap()
                .id,
            id
        );

        //Test for price <= 0 and amount <= 0
        id += 1;
        assert_eq!(
            insert_limit(
                &mut orderbook,
                &id,
                AskOrBid::Bid,
                &Decimal::from(0),
                &unfilled
            ),
            InsertLimitResult::OrderDataInvalid
        );
        id += 1;
        assert_eq!(
            insert_limit(
                &mut orderbook,
                &id,
                AskOrBid::Bid,
                &Decimal::from(-1),
                &unfilled
            ),
            InsertLimitResult::OrderDataInvalid
        );
        id += 1;
        assert_eq!(
            insert_limit(
                &mut orderbook,
                &id,
                AskOrBid::Bid,
                &price,
                &Decimal::from(-1)
            ),
            InsertLimitResult::OrderDataInvalid
        );
        id += 1;
        assert_eq!(
            insert_limit(
                &mut orderbook,
                &id,
                AskOrBid::Bid,
                &price,
                &Decimal::from(0)
            ),
            InsertLimitResult::OrderDataInvalid
        );
    }

    #[test]
    fn test_get_best_ask_bid() {
        let mut orderbook = Orderbook::new(Symbol::BTC);
        let mut id = 0u64;
        let amount = Decimal::from(50);

        //Change seed used for rng here
        let mut gen = rand_chacha::ChaCha8Rng::seed_from_u64(39);

        //No order has been written yet
        assert_eq!(orderbook.get_best_ask(), None);
        assert_eq!(orderbook.get_best_bid(), None);

        //Add a bunch of orders at random prices to ask and bid and save the best ask and bid price
        //for comparison
        let mut best_ask = Decimal::from(-1);
        let mut best_bid = Decimal::from(-1);

        for _n in 1..10 {
            let rand_price = Decimal::from(gen.next_u32() % 100 + 512);
            if rand_price < best_ask || best_ask.is_sign_negative() {
                best_ask = rand_price;
            }
            if rand_price > best_bid || best_bid.is_sign_negative() {
                best_bid = rand_price;
            }

            insert_limit(&mut orderbook, &id, AskOrBid::Ask, &rand_price, &amount);
            id += 1;
            insert_limit(&mut orderbook, &id, AskOrBid::Bid, &rand_price, &amount);
            id += 1;
        }
        assert_eq!(orderbook.get_best_ask().unwrap(), best_ask);
        assert_eq!(orderbook.get_best_bid().unwrap(), best_bid);
    }

    #[test]
    fn test_contains_order() {
        let mut orderbook = Orderbook::new(Symbol::BTC);
        let id = 1234u64;

        assert_eq!(orderbook.contains_order(&0), false);
        assert_eq!(orderbook.contains_order(&1342), false);

        insert_limit(
            &mut orderbook,
            &id,
            AskOrBid::Bid,
            &Decimal::from(328),
            &Decimal::from(834),
        );
        assert_eq!(orderbook.contains_order(&id), true);
    }

    #[test]
    fn test_can_match() {
        let mut orderbook = Orderbook::new(Symbol::BTC);
        let amount = Decimal::from(3945);

        assert_eq!(orderbook.can_match(), false);

        insert_limit(
            &mut orderbook,
            &0u64,
            AskOrBid::Bid,
            &Decimal::from(500),
            &amount,
        );
        assert_eq!(orderbook.can_match(), false);
        insert_limit(
            &mut orderbook,
            &1u64,
            AskOrBid::Ask,
            &Decimal::from(501),
            &amount,
        );

        assert_eq!(orderbook.can_match(), false);

        insert_limit(
            &mut orderbook,
            &2u64,
            AskOrBid::Bid,
            &Decimal::from(501),
            &amount,
        );

        assert_eq!(orderbook.can_match(), true);
    }

    #[test]
    fn test_get_orderbook_side_for_price() {
        let mut orderbook = Orderbook::new(Symbol::BTC);
        let price_ask = Decimal::from(510);
        let price_bid = Decimal::from(505);
        let amount = Decimal::from(3945);

        //No orders in orderbook
        assert_eq!(
            orderbook.get_side_for_price(&price_ask).is_none(),
            true
        );

        //check for asks
        insert_limit(&mut orderbook, &0, AskOrBid::Ask, &price_ask, &amount);

        assert_eq!(
            orderbook
                .get_side_for_price(&(price_ask - Decimal::from(1)))
                .is_none(),
            true
        );
        assert_eq!(orderbook.get_side_for_price(&price_ask).unwrap(), AskOrBid::Ask);

        //Check for bids
        insert_limit(&mut orderbook, &1, AskOrBid::Bid, &price_bid, &amount);

        assert_eq!(
            orderbook
                .get_side_for_price(&(price_bid + Decimal::from(1)))
                .is_none(),
            true
        );
        assert_eq!(orderbook.get_side_for_price(&price_bid).unwrap(), AskOrBid::Bid);

        //Bring orderbook into inconsistent state
        insert_limit(&mut orderbook, &2, AskOrBid::Bid, &price_ask, &amount);
        assert_eq!(
            orderbook.get_side_for_price(&price_ask).is_none(),
            true
        );
    }

    #[test]
    fn test_get_order_mut() {
        let mut orderbook = Orderbook::new(Symbol::BTC);
        let price = Decimal::from(505);
        let amount = Decimal::from(3945);

        assert_eq!(orderbook.get_order_mut(&0).is_none(), true);

        insert_limit(&mut orderbook, &432, AskOrBid::Bid, &price, &amount);
        insert_limit(&mut orderbook, &2130, AskOrBid::Ask, &(price + Decimal::from(1)), &amount);

        assert_eq!(orderbook.get_order_mut(&432).unwrap().unfilled, amount);
        assert_eq!(orderbook.get_order_mut(&212).is_none(), true);
    }

    #[test]
    fn test_cancel_limit() {
        let mut orderbook = Orderbook::new(Symbol::BTC);

        assert_eq!(orderbook.cancel_limit(&0), CancelLimitResult::OrderIdNotFound);
        insert_limit(
            &mut orderbook,
            &0,
            AskOrBid::Bid,
            &Decimal::from(20),
            &Decimal::from(20),
        );

        assert_eq!(orderbook.cancel_limit(&0), CancelLimitResult::Success);
        assert_eq!(orderbook.get_best_bid(), None);
    }
}
