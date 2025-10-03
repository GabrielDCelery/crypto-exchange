use std::collections::HashMap;

use time::OffsetDateTime;
use uuid::Uuid;

// struct Match {
//     ask_id: Uuid,
//     bid_id: Uuid,
//     size_filled: f64,
//     price: f64,
// }

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
enum OrderType {
    Bid,
    Ask,
}

struct Order {
    id: Uuid,
    order_type: OrderType,
    size: f64,
    timestamp: i64,
    limit_id: Option<Uuid>,
}

impl Order {
    fn new(order_type: OrderType, size: f64) -> Self {
        return Order {
            id: Uuid::new_v4(),
            order_type: order_type,
            size,
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
            limit_id: None,
        };
    }
}

// A group of orders at a certain price level
// a bucket of orders that are of different sizes sitting on the same price level
struct Limit {
    id: Uuid,
    price: f64,
    orders: Vec<Order>,
    total_volume: f64,
}

impl Limit {
    fn new(price: f64) -> Limit {
        return Limit {
            id: Uuid::new_v4(),
            price,
            orders: vec![],
            total_volume: 0.0,
        };
    }

    fn add_order(&mut self, mut o: Order) {
        o.limit_id = Some(self.id);
        self.total_volume += o.size;
        self.orders.push(o);
    }

    fn remove_order(&mut self, order_id: Uuid) -> Result<(), String> {
        let index = self.orders.iter().position(|x: &Order| x.id == order_id);
        match index {
            Some(i) => {
                let mut removed_order = self.orders.swap_remove(i);
                removed_order.limit_id = None;
                self.total_volume -= removed_order.size;
                Ok(())
            }
            None => Err(format!("Could not find order by id {order_id}")),
        }
    }
}

struct OrderBook {
    limits: HashMap<OrderType, Vec<Limit>>,
    limits_by_price: HashMap<OrderType, HashMap<String, usize>>,
}

impl OrderBook {
    fn add_order(&mut self, price: f64, order: Order) -> Result<(), String> {
        let price_key = price.to_string();

        match (
            self.limits.get_mut(&order.order_type),
            self.limits_by_price.get_mut(&order.order_type),
        ) {
            (Some(limits), Some(limits_by_price)) => match limits_by_price.get(&price_key) {
                Some(&limit_idx) => {
                    let next_limit_idx = limits.len();
                    match limits.get_mut(limit_idx) {
                        Some(limit) => {
                            limit.add_order(order);
                            limits_by_price.insert(price_key, next_limit_idx);
                            return Ok(());
                        }
                        None => {
                            return Err(format!(""));
                        }
                    }
                }
                None => {
                    let mut limit = Limit::new(price);
                    limit.add_order(order);
                    limits.push(limit);
                    limits_by_price.insert(price_key, 0);
                    return Ok(());
                }
            },
            (_, _) => {
                return Err(format!("Invalid order type"));
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::order_book::{Limit, Order, OrderType};

    #[test]
    fn successfully_adds_a_buy_order_to_a_limit() {
        // Given
        let mut limit = Limit::new(10_000.0);
        let buy_order = Order::new(OrderType::Bid, 5.0);

        // When
        limit.add_order(buy_order);

        // Then
        assert_eq!(limit.orders.len(), 1);
    }

    #[test]
    fn successfully_removes_a_buy_order_from_a_limit() {
        // Given
        let mut limit = Limit::new(10_000.0);
        let buy_order_a = Order::new(OrderType::Bid, 5.0);
        let buy_order_b = Order::new(OrderType::Bid, 8.0);
        let buy_order_c = Order::new(OrderType::Bid, 10.0);

        // Store the ID before moving the order
        let buy_order_b_id = buy_order_b.id;

        limit.add_order(buy_order_a);
        limit.add_order(buy_order_b);
        limit.add_order(buy_order_c);

        // When
        let _result = limit.remove_order(buy_order_b_id);

        // Then
        assert_eq!(limit.total_volume, 15.0);
    }
}
