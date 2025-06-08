use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default)]
pub struct UserHoldings {
    pub id: Uuid,
    pub user_id: Uuid,
    pub market_id: Uuid,
    pub shares: Decimal,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl UserHoldings {
    pub async fn create_user_holding(
        pg_pool: &sqlx::PgPool,
        user_id: Uuid,
        market_id: Uuid,
        shares: Decimal,
    ) -> Result<UserHoldings, sqlx::error::Error> {
        let holding = sqlx::query_as!(
            UserHoldings,
            r#"
            INSERT INTO polymarket.user_holdings (user_id, market_id, shares)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, market_id, shares, created_at, updated_at
            "#,
            user_id,
            market_id,
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
        quantity: Decimal,
    ) -> Result<UserHoldings, sqlx::error::Error> {
        let holding = sqlx::query_as!(
            UserHoldings,
            r#"
            INSERT INTO polymarket.user_holdings (user_id, market_id, shares)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, market_id)
            DO UPDATE SET shares = polymarket.user_holdings.shares + $3,
            updated_at = NOW()
            RETURNING id, user_id, market_id, shares, created_at, updated_at;
            "#,
            user_id,
            market_id,
            quantity
        )
        .fetch_optional(db_pool)
        .await?;

        if holding.is_none() {
            let holdings = Self::create_user_holding(db_pool, user_id, market_id, quantity).await?;
            return Ok(holdings);
        }

        Ok(holding.unwrap())
    }

    pub async fn get_user_holdings(
        db_pool: &sqlx::PgPool,
        user_id: Uuid,
        market_id: Uuid,
    ) -> Result<UserHoldings, sqlx::error::Error> {
        let mut tx = db_pool.begin().await?;

        // making sure the user holding exists, as on initial new order creation, it might not exist
        sqlx::query!(
            r#"
            INSERT INTO polymarket.user_holdings (user_id, market_id, shares)
            VALUES ($1, $2, 0)
            ON CONFLICT (user_id, market_id) DO NOTHING
            "#,
            user_id,
            market_id
        )
        .execute(&mut *tx)
        .await?;

        let holdings = sqlx::query_as!(
            UserHoldings,
            r#"
            SELECT id, user_id, market_id, shares, created_at, updated_at
            FROM polymarket.user_holdings
            WHERE user_id = $1 AND market_id = $2
            "#,
            user_id,
            market_id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(holdings)
    }
}
