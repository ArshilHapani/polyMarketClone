pub enum KafkaTopics {
    PriceUpdates,
    MarketOrderBook(String),
}

impl KafkaTopics {
    pub fn from_str(topic: &str) -> Option<Self> {
        if topic.starts_with("market-order-book-") {
            let market_id = topic.trim_start_matches("market-order-book-").to_string();
            Some(KafkaTopics::MarketOrderBook(market_id))
        } else if topic == "price-updates" {
            Some(KafkaTopics::PriceUpdates)
        } else {
            None
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            KafkaTopics::PriceUpdates => "price-updates".to_string(),
            KafkaTopics::MarketOrderBook(market_id) => format!("market-order-book-{}", market_id),
        }
    }
}
