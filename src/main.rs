use std::collections::{BTreeMap, VecDeque};

use alloy::primitives::U256;

enum Side {
    Bid,
    Ask,
}

struct Order {
    owner: String,
    nonce: U256,
    quantity: U256,
    stop_price: U256,
    expire_timestamp: u64,
    side: Side,
    only_full_fill: bool,
}

struct OrderBook {
    bids: BTreeMap<U256, VecDeque<Order>>,
    asks: BTreeMap<U256, VecDeque<Order>>,
    
}

impl OrderBook {
    fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    fn add_order(&mut self, order: Order) {
        let side = order.side;
        let price = order.limit_price;
        let queue = match side {
            Side::Bid => self.bids,
            Side::Ask => self.asks,
        }
        .entry(price)
        .or_insert_with(VecDeque::new);
        queue.push_back(order);
    }
}

fn main() {
    println!("Hello, world!");
}
