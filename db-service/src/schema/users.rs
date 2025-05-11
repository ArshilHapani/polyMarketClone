use base64::{Engine, engine::general_purpose::STANDARD as base64_engine};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::{signature::Keypair, signer::Signer};
use sqlx::PgPool;
use uuid::Uuid;

use crate::log_info;
use auth_service::{symmetric::encrypt, types::GoogleClaims};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct User {
    pub id: Uuid,

    // oAuth2 fields
    pub google_id: String,
    pub email: String,
    pub name: String,
    pub avatar: String,
    pub last_login: NaiveDateTime,
    pub refresh_token: String,

    // wallet fields
    pub public_key: String,
    pub private_key: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub balance: Decimal,
}

impl User {
    pub async fn create_new_user(pool: &PgPool, claims: GoogleClaims) -> Result<Self, sqlx::Error> {
        let new_key_pair = Keypair::new();

        let private_key = new_key_pair.to_base58_string();
        let public_key = new_key_pair.pubkey().to_string();

        let encrypted_private_key_bytes = encrypt(private_key.as_bytes())
            .map_err(|_| sqlx::Error::Decode("Failed to encrypt private key".into()))?;
        let encrypted_private_key = base64_engine.encode(encrypted_private_key_bytes);

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO "polymarket"."users" (
                google_id,
                email,
                name,
                avatar,
                public_key, 
                private_key
            ) VALUES (
                $1, $2, $3, $4, $5, $6
            ) RETURNING *
            "#,
            claims.sub,
            claims.email,
            claims.name,
            claims.picture,
            public_key,
            encrypted_private_key
        )
        .fetch_one(pool)
        .await?;

        log_info!("User added {}", user.id);

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auth_service::symmetric::decrypt;
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

        let decoded_private_key = base64_engine.decode(&user.private_key).unwrap();
        let decrypted_private_key = decrypt(&decoded_private_key).unwrap();
        let _decrypted_private_key_str = String::from_utf8(decrypted_private_key).unwrap();

        assert_eq!(user.public_key.len(), 44);
        assert_eq!(user.private_key.len(), 156);
        assert_eq!(user.balance, Decimal::ZERO);
        assert_eq!(user.created_at, user.updated_at);

        // Clean up
        sqlx::query(r#"DELETE FROM "polymarket"."users" WHERE id = $1"#)
            .bind(user.id)
            .execute(&pool)
            .await
            .unwrap();

        pool.close().await;
    }
}
