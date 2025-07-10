use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::schema::enums::OrderSide;

use super::enums::Outcome;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default)]
pub struct UserTrades {
    id: Uuid,
    // TODO: in free time change the field names `buy_order_id` and `sell_order_id` to `current_order_id` and `opposite_order_id`
    buy_order_id: Uuid,
    sell_order_id: Uuid,
    user_id: Uuid,
    market_id: Uuid,
    trade_type: OrderSide,
    outcome: Outcome,
    price: Decimal,
    quantity: Decimal,
    timestamp: NaiveDateTime,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl UserTrades {
    pub async fn create_user_trade<'a>(
        executor: impl Executor<'a, Database = Postgres>,
        current_order_id: Uuid,
        opposite_order_id: Uuid,
        user_id: Uuid,
        market_id: Uuid,
        outcome: Outcome,
        price: Decimal,
        quantity: Decimal,
        trade_type: OrderSide,
    ) -> Result<UserTrades, sqlx::error::Error> {
        let trade = sqlx::query_as!(
            UserTrades,
            r#"
            INSERT INTO polymarket.user_trades (buy_order_id, sell_order_id, user_id, market_id, outcome, price, quantity, trade_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, buy_order_id, sell_order_id, user_id, market_id,
            outcome as "outcome: Outcome",
            price, quantity, timestamp, created_at, updated_at,
            trade_type as "trade_type: OrderSide"
            "#,
            current_order_id,
            opposite_order_id,
            user_id,
            market_id,
            outcome as Outcome,
            price,
            quantity,
            trade_type as OrderSide,
        ).fetch_one(executor).await?;
        Ok(trade)
    }
}
