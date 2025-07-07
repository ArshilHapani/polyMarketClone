use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Postgres};
use uuid::Uuid;

use crate::schema::enums::Outcome;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default)]
pub struct UserHoldings {
    pub id: Uuid,
    pub user_id: Uuid,
    pub market_id: Uuid,
    pub shares: Decimal,
    pub outcome: Outcome,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl UserHoldings {
    pub async fn create_user_holding<'a>(
        executor: impl Executor<'a, Database = Postgres>,
        user_id: Uuid,
        market_id: Uuid,
        shares: Decimal,
        outcome: Outcome,
    ) -> Result<UserHoldings, sqlx::error::Error> {
        let holding = sqlx::query_as!(
            UserHoldings,
            r#"
            INSERT INTO polymarket.user_holdings (user_id, market_id, shares, outcome)
            VALUES ($1, $2, $3, $4)
            RETURNING 
                id, 
                user_id, 
                market_id, 
                shares, 
                created_at, 
                updated_at, 
                outcome as "outcome: Outcome";
            "#,
            user_id,
            market_id,
            shares,
            outcome as _
        )
        .fetch_one(executor)
        .await?;

        Ok(holding)
    }

    pub async fn update_user_holdings<'a>(
        executor: impl Executor<'a, Database = Postgres>,
        user_id: Uuid,
        market_id: Uuid,
        quantity: Decimal,
        outcome: Outcome,
    ) -> Result<UserHoldings, sqlx::error::Error> {
        let holding = sqlx::query_as!(
            UserHoldings,
            r#"
            INSERT INTO polymarket.user_holdings (user_id, market_id, shares, outcome)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, market_id, outcome)
            DO UPDATE SET shares = polymarket.user_holdings.shares + $3,
            updated_at = NOW()
            RETURNING 
                id, 
                user_id, 
                market_id, 
                shares, 
                created_at, 
                updated_at, 
                outcome as "outcome: Outcome";
            "#,
            user_id,
            market_id,
            quantity,
            outcome as _
        )
        .fetch_one(executor)
        .await?;

        Ok(holding)
    }

    pub async fn get_user_holdings_by_outcome(
        db_pool: &sqlx::PgPool,
        user_id: Uuid,
        market_id: Uuid,
        outcome: Outcome,
    ) -> Result<UserHoldings, sqlx::error::Error> {
        let mut tx = db_pool.begin().await?;
        println!("Outcome: {:?}", outcome);

        // making sure the user holding exists, as on initial new order creation, it might not exist
        sqlx::query!(
            r#"
            INSERT INTO polymarket.user_holdings (user_id, market_id, shares, outcome)
            VALUES ($1, $2, 0, $3)
            ON CONFLICT (user_id, market_id, outcome) DO NOTHING
            "#,
            user_id,
            market_id,
            outcome as _
        )
        .execute(&mut *tx)
        .await?;

        let holdings = sqlx::query_as!(
            UserHoldings,
            r#"
            SELECT id, user_id, market_id, shares, created_at, updated_at, outcome as "outcome: Outcome"
            FROM polymarket.user_holdings
            WHERE user_id = $1 AND market_id = $2 AND outcome = $3
            "#,
            user_id,
            market_id,
            outcome as _
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(holdings)
    }

    pub async fn get_user_holdings(
        db_pool: &sqlx::PgPool,
        user_id: Uuid,
        market_id: Uuid,
    ) -> Result<Vec<UserHoldings>, sqlx::error::Error> {
        sqlx::query_as!(
            UserHoldings,
            r#"
            SELECT id, user_id, market_id, shares, created_at, updated_at, outcome as "outcome: Outcome"
            FROM polymarket.user_holdings
            WHERE user_id = $1 AND market_id = $2
            "#,
            user_id,
            market_id
        )
        .fetch_all(db_pool)
        .await
    }

    pub async fn get_user_holdings_sum_both_outcome(
        db_pool: &sqlx::PgPool,
        user_id: Uuid,
        market_id: Uuid,
    ) -> Result<(Decimal, Decimal), sqlx::error::Error> {
        let result = sqlx::query!(
            r#"
            SELECT 
                SUM(CASE WHEN outcome = 'yes'::polymarket.outcome THEN shares ELSE 0 END) as "yes_shares",
                SUM(CASE WHEN outcome = 'no'::polymarket.outcome THEN shares ELSE 0 END) as "no_shares"
            FROM polymarket.user_holdings
            WHERE user_id = $1 AND market_id = $2
            "#,
            user_id,
            market_id
        )
        .fetch_one(db_pool)
        .await?;

        let yes_shares = result.yes_shares.unwrap_or(Decimal::ZERO);
        let no_shares = result.no_shares.unwrap_or(Decimal::ZERO);
        Ok((yes_shares, no_shares))
    }
}
