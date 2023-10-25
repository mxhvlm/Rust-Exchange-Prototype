use core::num;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use exchange_prototype::order_matcher::OrderMatcher;
use exchange_prototype::order_matcher_fifo::OrderMatcherFifo;
use exchange_prototype::orderbook::Orderbook;
use exchange_prototype::symbol::{AskOrBid, Symbol};
use rand::distributions::Distribution;
use rand::prelude::StdRng;
use rand::{thread_rng, RngCore, SeedableRng};
use rand_distr::Normal;
use rust_decimal::Decimal;

fn test_match_limit_performance(
    price_levels_buy: &Vec<Decimal>,
    price_levels_sell: &Vec<Decimal>,
    orderbook: &mut Orderbook,
    matcher: &OrderMatcherFifo,
    iterations: u64,
) -> usize {
    let mut rand = 0;
    let len_buy = price_levels_buy.len();
    let len_sell = price_levels_sell.len();
    let mut rng = StdRng::seed_from_u64(42);

    for i in 0u64..iterations {
        rand = rng.next_u32() as usize;

        let result = match i % 2 == 0 {
            true => matcher.match_limit(
                orderbook,
                &i,
                AskOrBid::Bid,
                &price_levels_buy[rand % len_buy],
                &price_levels_buy[rand % len_buy],
            ),
            false => matcher.match_limit(
                orderbook,
                &i,
                AskOrBid::Ask,
                &price_levels_sell[rand % len_sell],
                &price_levels_sell[rand % len_sell],
            ),
        };
    }

    rand
}

pub fn criterion_benchmark(c: &mut Criterion) {
    print!("Running criterion benchnmark...");

    let num_orderbook_pages = vec![200, 2000, 20000, 200000];
    let transaction_counts = vec![10000, 100000, 1000000];
    let max_orderbook_pages = num_orderbook_pages.iter().max().unwrap();

    // Random sequences
    let mut rng_buy = StdRng::seed_from_u64(42);
    let mut rng_sell = StdRng::seed_from_u64(41);

    // Initialize orderbook and matcher
    let mut orderbook = Orderbook::new(Symbol::ETH);
    let matcher = OrderMatcherFifo::new();

    // Set initial market conditions
    let bid_ask_spread = 5.;
    let mean_price = 5012.;

    // Normal distributions for bid and ask orders
    // Mean for both differs by bid_ask_spread
    let normal_buy = Normal::new(mean_price, 100.0).unwrap();
    let normal_sell = Normal::new(mean_price + bid_ask_spread, 100.0).unwrap();

    let mut price_levels_buy: Vec<Decimal> = normal_buy
        .sample_iter(&mut rng_buy)
        .take(max_orderbook_pages / 2)
        .map(|x: f32| Decimal::from((x * 1f32) as u32))
        .filter(|x| x.is_sign_positive())
        .collect();
    let mut price_levels_sell: Vec<Decimal> = normal_sell
        .sample_iter(&mut rng_sell)
        .take(max_orderbook_pages / 2)
        .map(|x: f32| Decimal::from((x * 1f32) as u32))
        .filter(|x| x.is_sign_positive())
        .collect();

    for num_pages in &num_orderbook_pages {
        for num_transactions in &transaction_counts {
            price_levels_buy.sort();

            c.bench_function(
                &format!(
                    "Running test for {} discrete price levels with {}k transactions",
                    *num_pages, *num_transactions / 1000
                ),
                |b| {
                    b.iter(|| {
                        test_match_limit_performance(
                            black_box(&price_levels_buy.iter().take(*num_pages).copied().collect()),
                            black_box(&price_levels_sell.iter().take(*num_pages).copied().collect()),
                            &mut orderbook,
                            &matcher,
                            *num_transactions,
                        );
                        orderbook.orders_bid.clear();
                        orderbook.orders_ask.clear();
                        orderbook.orders_index.clear();
                    })
                },
            );
        }
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
