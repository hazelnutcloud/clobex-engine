use std::collections::{BTreeMap, VecDeque};

use alloy::primitives::U256;
use anyhow::{bail, Result};

#[derive(Clone)]
enum Side {
    Bid,
    Ask,
}

#[derive(Clone)]
struct Order {
    owner: String,
    nonce: U256,
    quantity: U256,
    filled_quantity: U256,
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

    fn take_bid_order(&mut self, cursor: usize) -> Option<(Order, Vec<Order>)> {
        // take the oldest market order
        let Some(taker_order) = self.market_bids.get_mut(cursor) else {
            // TODO: go through stop orders
            return None;
        };

        let mut taker_available_quantity = taker_order.quantity - taker_order.filled_quantity;
        let mut maker_orders: Vec<Order> = Vec::new();
        let mut empty_price_levels: Vec<U256> = Vec::new();

        // go through limit asks at each price level
        for (price_level, asks) in self.asks.iter_mut() {
            // go through each limit ask in this price level, oldest first
            let mut ask_cursor = 0;
            loop {
                match asks.get_mut(ask_cursor) {
                    Some(ask) => {
                        let ask_available_quantity = ask.quantity - ask.filled_quantity;
                        // if the ask order is only partially filled
                        if ask_available_quantity > taker_available_quantity {
                            if ask.only_full_fill {
                                ask_cursor += 1;
                                continue;
                            }
                            ask.filled_quantity += taker_available_quantity;
                            maker_orders.push(ask.clone());
                            taker_available_quantity = U256::ZERO;
                        } else {
                            // if the ask order is completely filled
                            maker_orders.push(asks.remove(ask_cursor).unwrap());
                            taker_available_quantity -= ask_available_quantity;
                        }
                    }
                    None => {
                        if ask_cursor == 0 {
                            empty_price_levels.push(*price_level);
                        }
                        break;
                    }
                }
                if taker_available_quantity == U256::ZERO {
                    break;
                }
            }
            if taker_available_quantity == U256::ZERO {
                break;
            }
        }

        for empty_price_level in empty_price_levels {
            self.asks.remove(&empty_price_level);
        }

        if taker_available_quantity > U256::ZERO {
            if taker_order.only_full_fill {
                return self.take_bid_order(cursor + 1);
            }
            taker_order.filled_quantity = taker_order.quantity - taker_available_quantity;
            if maker_orders.len() > 0 {
                return Some((taker_order.clone(), maker_orders));
            }
            return None;
        } else {
            if maker_orders.len() > 0 {
                return Some((self.market_bids.remove(cursor).unwrap(), maker_orders));
            }
            return None;
        }
    }
}

fn main() {
    println!("Hello, world!");
}
