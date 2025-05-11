use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::{signature::Keypair, signer::Signer};
use sqlx::PgPool;
use uuid::Uuid;

use crate::log_info;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct User {
    id: Uuid,
    public_key: String,
    private_key: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    balance: Decimal,
}

impl User {
    pub async fn create_new_user(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let new_key_pair = Keypair::new();
        let private_key = new_key_pair.to_base58_string();
        let public_key = new_key_pair.pubkey().to_string();

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO "polymarket"."users" (public_key, private_key) VALUES ($1, $2) RETURNING id, created_at, updated_at, public_key, balance, private_key
            "#,
            public_key,
            private_key
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

        let user = User::create_new_user(&pool).await.unwrap();

        assert_eq!(user.public_key.len(), 44);
        assert_eq!(user.private_key.len(), 88);
        assert_eq!(user.balance, Decimal::ZERO);
        assert!(user.created_at.and_utc().timestamp() > 0);
        assert!(user.updated_at.and_utc().timestamp() > 0);
        assert_eq!(user.created_at, user.updated_at);
        assert_eq!(user.id.to_string().len(), 36);

        // Clean up
        // sqlx::query(r#"DELETE FROM "polymarket"."users" WHERE id = $1"#)
        //     .bind(user.id)
        //     .execute(&pool)
        //     .await
        //     .unwrap();

        pool.close().await;
    }
}
