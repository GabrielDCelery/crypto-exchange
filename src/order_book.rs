use time::OffsetDateTime;
use uuid::Uuid;

enum Order {
    Bid { id: Uuid, size: f64, timestamp: i64 },
    Ask { id: Uuid, size: f64, timestamp: i64 },
}

impl Order {
    fn new_bid(size: f64) -> Self {
        return Order::Bid {
            id: Uuid::new_v4(),
            size,
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
        };
    }

    fn new_ask(size: f64) -> Self {
        return Order::Ask {
            id: Uuid::new_v4(),
            size,
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
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
}

// A group of orders at a certain price level
// a bucket of orders that are of different sizes sitting on the same price level
struct Limit<'a> {
    price: f64,
    orders: Vec<&'a Order>,
    total_volume: f64,
}

impl<'a> Limit<'a> {
    fn new(price: f64) -> Limit<'a> {
        return Limit {
            price,
            orders: vec![],
            total_volume: 0.0,
        };
    }

    fn add_order(&mut self, o: &'a Order) {
        self.total_volume += o.size();
        self.orders.push(o);
    }

    fn remove_order(&mut self, order_id: Uuid) -> Result<(), String> {
        let index = self.orders.iter().position(|x: &&Order| x.id() == order_id);
        match index {
            Some(i) => {
                let removed_order = self.orders.swap_remove(i);
                self.total_volume -= removed_order.size();
                Ok(())
            }
            None => Err(format!("Could not find order by id {order_id}")),
        }
    }
}

struct OrderBook<'a> {
    // an ask is a buy order for Bitcoin
    asks: Vec<Limit<'a>>,
    // a bid is a sell order for Bitcoin
    bids: Vec<Limit<'a>>,
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
        limit.add_order(&buy_order);

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

        limit.add_order(&buy_order_a);
        limit.add_order(&buy_order_b);
        limit.add_order(&buy_order_c);

        // When
        let result = limit.remove_order(buy_order_b.id());

        // Then
        assert_eq!(limit.total_volume, 15.0);
    }
}
