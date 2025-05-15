use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::enums::Outcome;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default)]
pub struct UserHoldings {
    pub id: Uuid,
    pub user_id: Uuid,
    pub market_id: Uuid,
    pub outcome: Outcome,
    pub shares: Decimal,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl UserHoldings {
    pub async fn create_user_holding(
        pg_pool: &sqlx::PgPool,
        user_id: Uuid,
        market_id: Uuid,
        outcome: Outcome,
        shares: Decimal,
    ) -> Result<UserHoldings, sqlx::error::Error> {
        let holding = sqlx::query_as!(
            UserHoldings,
            r#"
            INSERT INTO polymarket.user_holdings (user_id, market_id, outcome, shares)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, market_id, outcome as "outcome: Outcome", shares, created_at, updated_at
            "#,
            user_id,
            market_id,
            outcome as Outcome,
            shares
        )
        .fetch_one(pg_pool)
        .await?;

        Ok(holding)
    }
}
