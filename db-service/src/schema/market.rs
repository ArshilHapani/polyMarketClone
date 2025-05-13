use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::log_info;

use super::enums::{MarketStatus, Outcome};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Market {
    pub id: Uuid,
    pub name: String,
    pub logo: String,
    pub status: MarketStatus,
    pub liquidity_b: Decimal,
    pub final_outcome: Outcome,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Market {
    pub async fn new(
        name: String,
        description: String,
        logo: String,
        liquidity_b: Decimal,
        pg_pool: &PgPool,
    ) -> Result<Self, sqlx::Error> {
        let market = sqlx::query_as!(
            Market,
            r#"
            INSERT INTO "polymarket"."markets" (
                name,
                description,
                logo,
                liquidity_b 
            ) VALUES (
                $1,
                $2,
                $3,
                $4 
            ) RETURNING *
            "#,
            name,
            description,
            logo,
            liquidity_b
        )
        .fetch_one(pg_pool)
        .await?;

        log_info!("Market created: {}", market.id);
        Ok(market)
    }
}
