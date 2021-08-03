use crate::symbol::{Symbol, AskOrBid};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap};
use log::info;
use rust_decimal::prelude::Zero;

struct OrderbookPage {
    pub orders: HashMap<u64, Order>,
    pub amount: Decimal
}

pub struct Orderbook {
    symbol: Symbol,
    orders_ask: BTreeMap<Decimal, OrderbookPage>,
    orders_bid: BTreeMap<Decimal, OrderbookPage>,
    orders_index: HashMap<u64, Decimal>
}

#[derive(Clone, PartialEq, Debug)]
pub struct Order {
    pub id: u64,
    pub unfilled: Decimal,
}

impl OrderbookPage {
    fn new(order: Order) -> OrderbookPage {
        let mut orders = HashMap::<u64, Order>::new();
        let amount = order.unfilled;
        orders.insert(order.id, order.clone());
        OrderbookPage {
            orders,
            amount
        }
    }
}

impl Orderbook {
    pub fn new(symbol: Symbol) -> Orderbook {
        Orderbook{
            symbol,
            orders_ask: BTreeMap::<Decimal, OrderbookPage>::new(),
            orders_bid: BTreeMap::<Decimal, OrderbookPage>::new(),
            orders_index: HashMap::<u64, Decimal>::new()
        }
    }

    pub fn get_best_ask(&self) -> Option<Decimal> {
        self.orders_ask.iter().next().map(|(price, _)| *price)
    }

    pub fn get_best_bid(&self) -> Option<Decimal> {
        self.orders_bid.iter().rev().next().map(|(price, _)|*price)
    }

    pub fn contains_order(&self, order_id: &u64) -> bool {
        self.orders_index.contains_key(order_id)
    }

    //TODO: Think of another name that does a better job of explaining that it gets either the buy or sell order book
    /**
    Returns None if either no orders are in the orderbook or if the orderbook is in an inconsistent state
     */
    fn get_orderbook_for_price(&self, price: &Decimal) -> Option<&BTreeMap<Decimal, OrderbookPage>> {
        //Orderbook is in an inconsistent state eg. get_best_buy() >= get_best_bid()
        if self.can_trade() {
            return None;
        }
        if let Some(best_ask) = self.get_best_ask() {
            if *price >= best_ask {
                return Some(&self.orders_ask);
            }
        }
        if let Some(best_bid) = self.get_best_bid() {
            if *price <= best_bid {
                return Some(&self.orders_bid);
            }
        }
        None
    }

    // fn get_order(&self, order_id: &u64) -> Option<Order> {
    //     match self.contains_order(order_id) {
    //         true => {
    //             let price = self.orders_index.get(order_id);
    //
    //         }
    //     }
    // }
    //
    // pub fn get_unfilled(&self, order_id: &u64) -> Option<Decimal> {
    //     match self.contains_order(order_id) {
    //         true => {
    //             let page = self.orders_index.
    //         },
    //         false => None
    //     }
    // }

    fn can_trade(&self) -> bool {
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
        info!("Best Ask: {}", self.get_best_ask().unwrap_or(Decimal::from(-1)));
        info!("Best Bid: {}", self.get_best_bid().unwrap_or(Decimal::from(-1)));
    }

    pub fn process_limit(&mut self, order_id: &u64, side: AskOrBid, price: &Decimal, size: &Decimal) -> bool {
        let order_id = order_id.clone();
        let size = size.clone();
        let price = price.clone();

        if !self.insert_limit(order_id, side, price, size) {
            return false;
        }

        if self.can_trade() {
            info!("Executing trade...");
        }

        true
    }

    //TODO: Split into insert() and process_limit() which checks whether the order can be matched directly

    fn insert_limit(&mut self, order_id: u64, side: AskOrBid, price: Decimal, size: Decimal) -> bool {

        if price <= Decimal::zero() || size <= Decimal::zero() {
            return false;
        }

        //Return false if an order with the same id is already inserted into orderbook
        if self.orders_index.contains_key(&order_id){
            return false;
        }

        let mut order = Order{ id: order_id, unfilled: size};

        let mut orderbook = match side {
            AskOrBid::Ask => &mut self.orders_ask,
            AskOrBid::Bid => &mut self.orders_bid
        };

        orderbook.entry(price).and_modify(|page| {
            page.amount += order.unfilled;
            page.orders.insert(order_id, order.clone());
        }).or_insert_with(|| OrderbookPage::new(order));

        self.orders_index.insert(order_id, price);

        info!("Inserted order {} at price {}", order_id, price);
        self.log_best_ask_bid();

        true
    }

    fn remove_limit(&mut self, order_id: &u64) -> bool {
        false
    }
}

#[cfg(test)]
mod orderbook_tests {
    use super::*;
    use rand::Rng;

    //Wrapper for isolated testing of Orderbook::insert_limit()
    fn insert_limit(orderbook: &mut Orderbook, order_id: &u64, side: AskOrBid, price: &Decimal, size: &Decimal) -> bool {
        let order_id = order_id.clone();
        let size = size.clone();
        let price = price.clone();

        orderbook.insert_limit(order_id, side, price, size)
    }

    #[test]
    fn test_new_page() {
        let order = Order{ id: 0, unfilled: Decimal::from(10) };
        let mut page = OrderbookPage::new(order.clone());

        assert_eq!(page.orders.len(), 1);
        assert_eq!(order, page.orders.iter().next().map(|(_, order)|order.clone()).unwrap());

        assert_eq!(order.unfilled, page.amount);
    }

    #[test]
    fn test_orderbook_add_limit() {
        let mut orderbook = Orderbook::new(Symbol::BTC);
        let mut id = 16u64;
        let price = Decimal::from(100);
        let unfilled = Decimal::from(16);

        //Adding limit order with an unused order_id
        assert_eq!(insert_limit(&mut orderbook, &id, AskOrBid::Ask, &price, &unfilled), true);

        //Check if order got written into the BTree
        assert_eq!(orderbook.orders_ask.get(&price).unwrap().orders.get(&id).unwrap().id, id);

        //Check if index got written into HashMap
        assert_eq!(orderbook.orders_index.get(&id).unwrap(), &price);

        //Adding an order with the same order_id twice shouldn't be possible.
        assert_eq!(insert_limit(&mut orderbook,&id, AskOrBid::Ask, &price, &unfilled), false);

        //Check bid side (Don't hanve to check indicies since there is only one HashMap
        id += 1;
        assert_eq!(insert_limit(&mut orderbook,&id, AskOrBid::Bid, &price, &unfilled), true);
        assert_eq!(orderbook.orders_bid.get(&price).unwrap().orders.get(&id).unwrap().id, id);

        //Test for price <= 0 and amount <= 0
        id += 1;
        assert_eq!(insert_limit(&mut orderbook, &id, AskOrBid::Bid, &Decimal::from(0), &unfilled), false);
        id += 1;
        assert_eq!(insert_limit(&mut orderbook, &id, AskOrBid::Bid, &Decimal::from(-1), &unfilled), false);
        id += 1;
        assert_eq!(insert_limit(&mut orderbook, &id, AskOrBid::Bid, &price, &Decimal::from(-1)), false);
        id += 1;
        assert_eq!(insert_limit(&mut orderbook, &id, AskOrBid::Bid, &price, &Decimal::from(0)), false);
    }

    #[test]
    fn test_get_best_ask_bid() {
        let mut orderbook = Orderbook::new(Symbol::BTC);
        let mut id = 0u64;
        let amount = Decimal::from(50);

        //No order has been written yet
        assert_eq!(orderbook.get_best_ask(), None);
        assert_eq!(orderbook.get_best_bid(), None);

        //Add a bunch of orders at random prices to ask and bid and save the best ask and bid price
        //for comparison
        let mut best_ask = Decimal::from(rand::thread_rng().gen_range(0..100) + 512);
        let mut best_bid = best_ask.clone();
        for n in 1..10 {
            let rand_price = Decimal::from(rand::thread_rng().gen_range(0..100) + 512);
            if rand_price < best_ask {
                best_ask = rand_price;
            }
            if rand_price > best_bid {
                best_bid = rand_price;
            }
            insert_limit(&mut orderbook,&id, AskOrBid::Ask, &rand_price, &amount);
            id += 1;
            insert_limit(&mut orderbook,&id, AskOrBid::Bid, &rand_price, &amount);
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

        insert_limit(&mut orderbook, &id, AskOrBid::Bid, &Decimal::from(328), &Decimal::from(834));
        assert_eq!(orderbook.contains_order(&id), true);
    }

    #[test]
    fn test_can_trade() {
        let mut orderbook = Orderbook::new(Symbol::BTC);
        let amount = Decimal::from(3945);

        assert_eq!(orderbook.can_trade(), false);

        insert_limit(&mut orderbook,&0u64, AskOrBid::Bid, &Decimal::from(500), &amount);
        assert_eq!(orderbook.can_trade(), false);
        insert_limit(&mut orderbook,&1u64, AskOrBid::Ask, &Decimal::from(501), &amount);

        assert_eq!(orderbook.can_trade(), false);

        insert_limit(&mut orderbook,&2u64, AskOrBid::Bid, &Decimal::from(501), &amount);

        assert_eq!(orderbook.can_trade(), true);
    }

    #[test]
    fn test_remove_limit() {
        let mut orderbook = Orderbook::new(Symbol::BTC);

        assert_eq!(orderbook.remove_limit(&0), false);
        insert_limit(&mut orderbook, &0, AskOrBid::Bid, &Decimal::from(20), &Decimal::from(20));
        //Check if you can remove order
        assert_eq!(orderbook.remove_limit(&0), true);
        assert_eq!(orderbook.get_best_bid(), None);
    }

}