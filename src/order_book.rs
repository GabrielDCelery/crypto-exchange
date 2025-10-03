use time::OffsetDateTime;
use uuid::Uuid;

enum Order {
    Bid {
        id: Uuid,
        size: f64,
        timestamp: i64,
        limit_id: Option<Uuid>,
    },
    Ask {
        id: Uuid,
        size: f64,
        timestamp: i64,
        limit_id: Option<Uuid>,
    },
}

impl Order {
    fn new_bid(size: f64) -> Self {
        return Order::Bid {
            id: Uuid::new_v4(),
            size,
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
            limit_id: None,
        };
    }

    fn new_ask(size: f64) -> Self {
        return Order::Ask {
            id: Uuid::new_v4(),
            size,
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
            limit_id: None,
        };
    }

    fn id(&self) -> Uuid {
        match self {
            Order::Bid { id, .. } => *id,
            Order::Ask { id, .. } => *id,
        }
    }

    fn size(&self) -> f64 {
        match self {
            Order::Bid { size, .. } => *size,
            Order::Ask { size, .. } => *size,
        }
    }

    fn set_limit_id(&mut self, new_limit_id: Option<Uuid>) {
        match self {
            Order::Bid { limit_id, .. } => {
                *limit_id = new_limit_id;
            }
            Order::Ask { limit_id, .. } => {
                *limit_id = new_limit_id;
            }
        }
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
        o.set_limit_id(Some(self.id));
        self.total_volume += o.size();
        self.orders.push(o);
    }

    fn remove_order(&mut self, order_id: Uuid) -> Result<(), String> {
        let index = self.orders.iter().position(|x: &Order| x.id() == order_id);
        match index {
            Some(i) => {
                let mut removed_order = self.orders.swap_remove(i);
                removed_order.set_limit_id(None);
                self.total_volume -= removed_order.size();
                Ok(())
            }
            None => Err(format!("Could not find order by id {order_id}")),
        }
    }
}

struct OrderBook {
    // an ask is a buy order for Bitcoin
    asks: Vec<Limit>,
    // a bid is a sell order for Bitcoin
    bids: Vec<Limit>,
}

#[cfg(test)]
pub mod tests {
    use crate::order_book::{Limit, Order};

    #[test]
    fn successfully_adds_a_buy_order_to_a_limit() {
        // Given
        let mut limit = Limit::new(10_000.0);
        let buy_order = Order::new_bid(5.0);

        // When
        limit.add_order(buy_order);

        // Then
        assert_eq!(limit.orders.len(), 1);
    }

    #[test]
    fn successfully_removes_a_buy_order_from_a_limit() {
        // Given
        let mut limit = Limit::new(10_000.0);
        let buy_order_a = Order::new_bid(5.0);
        let buy_order_b = Order::new_bid(8.0);
        let buy_order_c = Order::new_bid(10.0);

        // Store the ID before moving the order
        let buy_order_b_id = buy_order_b.id();

        limit.add_order(buy_order_a);
        limit.add_order(buy_order_b);
        limit.add_order(buy_order_c);

        // When
        let _result = limit.remove_order(buy_order_b_id);

        // Then
        assert_eq!(limit.total_volume, 15.0);
    }
}
