use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::log_info;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct User {
    id: Uuid,
    public_address: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    balance: Decimal,
}

impl User {
    pub async fn create_new_user(pool: &PgPool, address: String) -> Result<Self, sqlx::Error> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO "polymarket"."users" (public_address) VALUES ($1) RETURNING id, created_at, updated_at, public_address, balance
            "#,
            address
        ).fetch_one(pool).await?;

        log_info!("User added {}", user.id);

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use std::env;
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    fn setup_tracing() {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global default subscriber");
    }

    #[tokio::test]
    async fn test_create_new_user() {
        setup_tracing();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        let address = "0x1234567890abcdef".to_string();
        let user = User::create_new_user(&pool, address.clone()).await.unwrap();

        assert_eq!(user.public_address, address);

        // Clean up
        sqlx::query("DELETE FROM \"polymarket\".\"users\" WHERE id = $1")
            .bind(user.id)
            .execute(&pool)
            .await
            .unwrap();

        pool.close().await;
    }
}
