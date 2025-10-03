use std::collections::HashMap;
use std::fmt;

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

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self,)
    }
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

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(id: {}, order_type: {}, size: {}, timestamp: {}",
            self.id, self.order_type, self.size, self.timestamp
        )
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

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(id: {}, price: {}, total_volume: {})",
            self.id, self.price, self.total_volume
        )
    }
}

struct OrderBook {
    limits: HashMap<OrderType, Vec<Limit>>,
    limits_by_price: HashMap<OrderType, HashMap<String, usize>>,
}

impl OrderBook {
    fn new() -> Self {
        let mut limits: HashMap<OrderType, Vec<Limit>> = HashMap::new();
        let mut limits_by_price: HashMap<OrderType, HashMap<String, usize>> = HashMap::new();

        for e in vec![OrderType::Bid, OrderType::Ask] {
            limits.insert(e, vec![]);
            limits_by_price.insert(e, HashMap::new());
        }

        OrderBook {
            limits,
            limits_by_price,
        }
    }

    fn add_order(&mut self, price: f64, order: Order) -> Result<(), String> {
        let price_key = price.to_string();

        let limits = self
            .limits
            .get_mut(&order.order_type)
            .expect("Did not find limits for order type");

        let price_to_limit_idx_map = self
            .limits_by_price
            .get_mut(&order.order_type)
            .expect("Did not find limits by price for order type");

        match price_to_limit_idx_map.get(&price_key) {
            Some(&limit_idx) => {
                // We already have a limit for this price so we add the order to it
                if let Some(limit) = limits.get_mut(limit_idx) {
                    limit.add_order(order);
                    Ok(())
                } else {
                    Err(format!(
                        "Limit index {} is invalid for price {}",
                        limit_idx, price
                    ))
                }
            }
            None => {
                // These is no limit for this price yet so we need to create one
                let mut limit = Limit::new(price);
                limit.add_order(order);
                let new_limit_idx = limits.len();
                limits.push(limit);
                price_to_limit_idx_map.insert(price_key, new_limit_idx);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::order_book::{Limit, Order, OrderBook, OrderType};

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

    #[test]
    fn successfully_adds_a_buy_order_to_an_order_book() {
        //Given
        let mut order_book = OrderBook::new();
        let buy_order = Order::new(OrderType::Bid, 10.0);

        //When
        let _result = order_book.add_order(15_000.0, buy_order);

        //Then
        assert_eq!(order_book.limits.get(&OrderType::Bid).unwrap().len(), 1);
    }
}
