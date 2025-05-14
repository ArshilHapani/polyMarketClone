use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utility_helpers::log_info;
use uuid::Uuid;

use super::enums::{MarketStatus, Outcome};
use crate::pagination::PaginatedResponse;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default)]
pub struct Market {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub logo: String,
    pub status: MarketStatus,
    pub liquidity_b: Decimal,
    pub final_outcome: Outcome,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Market {
    pub async fn create_new_market(
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
            ) RETURNING 
                id,
                name,
                description,
                logo,
                status as "status: MarketStatus",
                final_outcome as "final_outcome: Outcome",
                liquidity_b,
                created_at,
                updated_at
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

    pub async fn get_all_markets(pg_pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let markets = sqlx::query_as!(
            Market,
            r#"
            SELECT 
                id,
                name,
                description,
                logo,
                status as "status: MarketStatus",
                final_outcome as "final_outcome: Outcome",
                liquidity_b,
                created_at,
                updated_at
            FROM "polymarket"."markets"
            "#,
        )
        .fetch_all(pg_pool)
        .await?;

        Ok(markets)
    }

    pub async fn get_all_markets_paginated(
        pg_pool: &PgPool,
        page: u64,
        page_size: u64,
    ) -> Result<PaginatedResponse<Self>, sqlx::Error> {
        let offset = (page - 1) * page_size;

        let total_count = sqlx::query!(
            r#"
            SELECT COUNT(*) as total_count
            FROM "polymarket"."markets"
            "#,
        )
        .fetch_one(pg_pool)
        .await?
        .total_count
        .unwrap_or(0);

        let total_pages = (total_count as u64 + page_size - 1) / page_size;

        let markets = sqlx::query_as!(
            Market,
            r#"
            SELECT 
                id,
                name,
                description,
                logo,
                status as "status: MarketStatus",
                final_outcome as "final_outcome: Outcome",
                liquidity_b,
                created_at,
                updated_at
            FROM "polymarket"."markets"
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            page_size as i64,
            offset as i64
        )
        .fetch_all(pg_pool)
        .await?;

        Ok(PaginatedResponse::new(
            markets,
            page,
            page_size,
            total_pages,
        ))
    }

    pub async fn get_market_by_id(
        pg_pool: &PgPool,
        market_id: Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        let market = sqlx::query_as!(
            Market,
            r#"
            SELECT 
                id,
                name,
                description,
                logo,
                status as "status: MarketStatus",
                final_outcome as "final_outcome: Outcome",
                liquidity_b,
                created_at,
                updated_at
            FROM "polymarket"."markets"
            WHERE id = $1
            "#,
            market_id
        )
        .fetch_optional(pg_pool)
        .await?;

        Ok(market)
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[tokio::test]
    async fn test_create_new_market() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pg_pool = PgPool::connect(&database_url).await.unwrap();

        let market = Market::create_new_market(
            "Test Market 0".to_string(),
            "Test Description".to_string(),
            "Test Logo".to_string(),
            Decimal::new(100, 2),
            &pg_pool,
        )
        .await
        .unwrap();

        assert_eq!(market.name, "Test Market 0");
        assert_eq!(market.description, "Test Description");
        assert_eq!(market.logo, "Test Logo");
        assert_eq!(market.liquidity_b, Decimal::new(100, 2));
        // Clean up the test market
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."markets" 
            WHERE id = $1
            "#,
            market.id
        )
        .execute(&pg_pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_get_all_markets() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pg_pool = PgPool::connect(&database_url).await.unwrap();

        let markets = Market::get_all_markets(&pg_pool).await;

        assert!(markets.is_ok());
    }

    #[tokio::test]
    async fn test_get_all_markets_paginated() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pg_pool = PgPool::connect(&database_url).await.unwrap();

        let paginated_response = Market::get_all_markets_paginated(&pg_pool, 1, 10)
            .await
            .unwrap();
        assert_eq!(paginated_response.page_info.page, 1);
        assert_eq!(paginated_response.page_info.page_size, 10);
    }

    #[tokio::test]
    async fn test_get_market_by_id() {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pg_pool = PgPool::connect(&database_url).await.unwrap();

        let market = Market::create_new_market(
            "Test Market".to_string(),
            "Test Description".to_string(),
            "Test Logo".to_string(),
            Decimal::new(100, 2),
            &pg_pool,
        )
        .await
        .unwrap();

        let fetched_market = Market::get_market_by_id(&pg_pool, market.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(fetched_market.id, market.id);
        assert_eq!(fetched_market.name, market.name);
        assert_eq!(fetched_market.description, market.description);
        assert_eq!(fetched_market.logo, market.logo);
        assert_eq!(fetched_market.liquidity_b, market.liquidity_b);

        // Clean up the test market
        sqlx::query!(
            r#"
            DELETE FROM "polymarket"."markets" 
            WHERE id = $1
            "#,
            market.id
        )
        .execute(&pg_pool)
        .await
        .unwrap();
    }
}
