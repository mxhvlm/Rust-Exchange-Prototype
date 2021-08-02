use crate::symbol::{Symbol, AskOrBid};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, LinkedList, HashMap};
use log::info;
use std::fmt;
use std::fmt::Formatter;

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

#[derive(Clone)]
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

    fn get_best_ask(&self) -> Option<Decimal> {
        self.orders_ask.iter().next().map(|(price, _)| *price)
    }

    fn get_best_bid(&self) -> Option<Decimal> {
        self.orders_bid.iter().rev().next().map(|(price, _)|*price)
    }

    fn can_trade(&self) -> bool {
        self.get_best_ask() <= self.get_best_bid()
    }

    fn log_best_ask_bid(&self) {
        info!("Best Ask: {}", self.get_best_ask().unwrap_or(Decimal::from(-1)));
        info!("Best Bid: {}", self.get_best_bid().unwrap_or(Decimal::from(-1)));
    }

    // pub fn try_exec_limit(&mut self, order_id: u64, side: AskOrBid, price: Decimal, size: Decimal)
    //     -> Option<Decimal>
    // {
    //     match side {
    //         AskOrBid::Ask => {
    //             if self.get_best_bid() >= price {
    //                 self.orders_bid.iter().rev().next()
    //             }
    //             else {
    //                 Some(size)
    //             }
    //         }
    //         AskOrBid::Bid => {
    //
    //         }
    //     }
    // }

    pub fn add_limit(&mut self, order_id: u64, side: AskOrBid, price: Decimal, size: Decimal) {
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

        if self.can_trade() {
            info!("Executing trade...");
        }
    }
}