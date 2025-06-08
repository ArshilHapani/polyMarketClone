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

    pub async fn update_user_holdings(
        db_pool: &sqlx::PgPool,
        user_id: Uuid,
        market_id: Uuid,
        outcome: Outcome,
        quantity: Decimal,
    ) -> Result<UserHoldings, sqlx::error::Error> {
        let holding = sqlx::query_as!(
            UserHoldings,
            r#"
            UPDATE polymarket.user_holdings
            SET shares = shares + $1
            WHERE user_id = $2 AND market_id = $3 AND outcome = $4
            RETURNING id, user_id, market_id, outcome as "outcome: Outcome", shares, created_at, updated_at
            "#,
            quantity,
            user_id,
            market_id,
            outcome as _
        ).fetch_optional(db_pool).await?;

        if holding.is_none() {
            let holdings =
                Self::create_user_holding(db_pool, user_id, market_id, outcome, quantity).await?;
            return Ok(holdings);
        }

        Ok(holding.unwrap())
    }

    pub async fn get_user_holdings(
        db_pool: &sqlx::PgPool,
        user_id: Uuid,
        market_id: Uuid,
    ) -> Result<Vec<UserHoldings>, sqlx::error::Error> {
        let holdings = sqlx::query_as!(
            UserHoldings,
            r#"
            SELECT id, user_id, market_id, outcome as "outcome: Outcome", shares, created_at, updated_at
            FROM polymarket.user_holdings
            WHERE user_id = $1 AND market_id = $2
            "#,
            user_id,
            market_id
        )
        .fetch_all(db_pool)
        .await?;

        Ok(holdings)
    }
}
