use std::collections::{BTreeMap, VecDeque};

use alloy::primitives::U256;
use anyhow::{bail, Result};

enum Side {
    Bid,
    Ask,
}

struct Order {
    owner: String,
    nonce: U256,
    quantity: U256,
    limit_price: U256,
    stop_price: U256,
    expire_timestamp: u64,
    side: Side,
    only_full_fill: bool,
}

enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

impl Order {
    fn order_type(&self) -> Option<OrderType> {
        match self.side {
            Side::Bid => match (self.limit_price, self.stop_price) {
                (U256::MAX, U256::ZERO) => Some(OrderType::Market),
                (limit_price, U256::ZERO) if limit_price < U256::MAX => Some(OrderType::Limit),
                (U256::MAX, stop_price) if stop_price > U256::ZERO => Some(OrderType::Stop),
                (limit_price, stop_price) if limit_price < U256::MAX && stop_price > U256::ZERO => {
                    Some(OrderType::StopLimit)
                }
                _ => None,
            },
            Side::Ask => match (self.limit_price, self.stop_price) {
                (U256::ZERO, U256::MAX) => Some(OrderType::Market),
                (limit_price, U256::MAX) if limit_price > U256::ZERO => Some(OrderType::Limit),
                (U256::ZERO, stop_price) if stop_price < U256::MAX => Some(OrderType::Stop),
                (limit_price, stop_price) if limit_price > U256::ZERO && stop_price < U256::MAX => {
                    Some(OrderType::StopLimit)
                }
                _ => None,
            },
        }
    }
}

struct OrderBook {
    bids: BTreeMap<U256, VecDeque<Order>>,
    asks: BTreeMap<U256, VecDeque<Order>>,
    stop_bids: BTreeMap<U256, VecDeque<Order>>,
    stop_asks: BTreeMap<U256, VecDeque<Order>>,
    market_bids: VecDeque<Order>,
    market_asks: VecDeque<Order>,
    last_price_level: U256,
}

impl OrderBook {
    fn from_initial_price(initial_price: U256) -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            stop_bids: BTreeMap::new(),
            stop_asks: BTreeMap::new(),
            market_bids: VecDeque::new(),
            market_asks: VecDeque::new(),
            last_price_level: initial_price,
        }
    }

    fn add_order(&mut self, order: Order) -> Result<()> {
        let Some(order_type) = order.order_type() else {
            bail!("Invalid order type");
        };
        match order_type {
            OrderType::Market => match order.side {
                Side::Bid => self.market_bids.push_back(order),
                Side::Ask => self.market_asks.push_back(order),
            },
            OrderType::Limit => match order.side {
                Side::Bid => self
                    .bids
                    .entry(order.limit_price)
                    .or_default()
                    .push_back(order),
                Side::Ask => self
                    .asks
                    .entry(order.limit_price)
                    .or_default()
                    .push_back(order),
            },
            OrderType::Stop | OrderType::StopLimit => match order.side {
                Side::Bid => self
                    .stop_bids
                    .entry(order.limit_price)
                    .or_default()
                    .push_back(order),
                Side::Ask => self
                    .stop_asks
                    .entry(order.limit_price)
                    .or_default()
                    .push_back(order),
            },
        }
        Ok(())
    }

    fn take_order(&mut self, side: Side) -> Option<(Order, Vec<Order>)> {
        match side {
            Side::Bid => todo!(),
            Side::Ask => todo!(),
        };
        None
    }
}

fn main() {
    println!("Hello, world!");
}
