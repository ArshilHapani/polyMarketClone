pub enum KafkaTopics {
    PriceUpdates,
    MarketOrderBookUpdate,
}

impl KafkaTopics {
    pub fn from_str(topic: &str) -> Option<Self> {
        if topic == "order-book-updates" {
            Some(KafkaTopics::MarketOrderBookUpdate)
        } else if topic == "price-updates" {
            Some(KafkaTopics::PriceUpdates)
        } else {
            None
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            KafkaTopics::PriceUpdates => "price-updates".to_string(),
            KafkaTopics::MarketOrderBookUpdate => "order-book-updates".to_string(),
        }
    }
}
