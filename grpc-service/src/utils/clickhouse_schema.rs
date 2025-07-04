use chrono::{DateTime, Utc};
use clickhouse::Row;
use serde::Deserialize;
use sqlx::types::Uuid;

#[derive(Row, Deserialize, Debug)]
pub struct GetMarketPrices {
    #[serde(with = "clickhouse::serde::uuid")]
    pub market_id: Uuid,
    #[serde(with = "clickhouse::serde::chrono::datetime")]
    pub ts: DateTime<Utc>,

    pub yes_price: f64,
    pub no_price: f64,
}

pub type OrderBook = Vec<(f64, f64, u32)>;

#[derive(Row, Deserialize, Debug)]
pub struct GetOrderBook {
    #[serde(with = "clickhouse::serde::uuid")]
    pub market_id: Uuid,
    #[serde(with = "clickhouse::serde::chrono::datetime")]
    pub ts: DateTime<Utc>,

    #[serde(with = "clickhouse::serde::chrono::datetime")]
    pub created_at: DateTime<Utc>,

    pub yes_bids: OrderBook,
    pub yes_asks: OrderBook,

    pub no_bids: OrderBook,
    pub no_asks: OrderBook,
}
