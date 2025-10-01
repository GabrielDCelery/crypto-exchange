use time::OffsetDateTime;

enum Order {
    Bid { size: f64, timestamp: i64 },
    Ask { size: f64, timestamp: i64 },
}

impl Order {
    fn new_bid(size: f64) -> Self {
        let now = OffsetDateTime::now_utc();
        let timestamp = now.unix_timestamp();
        return Order::Bid { size, timestamp };
    }

    fn new_ask(size: f64) -> Self {
        let now = OffsetDateTime::now_utc();
        let timestamp = now.unix_timestamp();
        return Order::Ask { size, timestamp };
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
struct Limit {
    price: f64,
    orders: Vec<Order>,
    total_volume: f64,
}

impl Limit {
    fn new(price: f64) -> Self {
        return Self {
            price,
            orders: vec![],
            total_volume: 0.0,
        };
    }

    fn add_order(&mut self, o: Order) {
        self.total_volume += o.size();
        self.orders.push(o);
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
}
