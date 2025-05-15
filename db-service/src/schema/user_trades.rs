use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::enums::Outcome;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default)]
pub struct UserTrades {
    id: Uuid,
    buy_order_id: Uuid,
    sell_order_id: Uuid,
    market_id: Uuid,
    outcome: Outcome,
    price: Decimal,
    quantity: Decimal,
    timestamp: NaiveDateTime,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl UserTrades {
    pub async fn create_user_trade(
        pg_pool: &PgPool,
        buy_order_id: Uuid,
        sell_order_id: Uuid,
        market_id: Uuid,
        outcome: Outcome,
        price: Decimal,
        quantity: Decimal,
    ) -> Result<UserTrades, sqlx::error::Error> {
        let trade = sqlx::query_as!(
            UserTrades,
            r#"
            INSERT INTO polymarket.user_trades (buy_order_id, sell_order_id, market_id, outcome, price, quantity)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, buy_order_id, sell_order_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, timestamp, created_at, updated_at
            "#,
            buy_order_id,
            sell_order_id,
            market_id,
            outcome as Outcome,
            price,
            quantity
        ).fetch_one(pg_pool).await?;

        Ok(trade)
    }
}
