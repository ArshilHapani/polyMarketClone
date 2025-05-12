use base64::{Engine, engine::general_purpose::STANDARD as base64_engine};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::{signature::Keypair, signer::Signer};
use sqlx::PgPool;
use uuid::Uuid;

use crate::log_info;
use utility_helpers::{symmetric::encrypt, types::GoogleClaims};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct User {
    pub id: Uuid,

    // oAuth2 fields
    pub google_id: String,
    pub email: String,
    pub name: String,
    pub avatar: String,
    pub last_login: NaiveDateTime,

    // wallet fields
    pub public_key: String,
    pub private_key: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub balance: Decimal,
}

impl User {
    pub async fn create_new_user(
        pool: &PgPool,
        claims: &GoogleClaims,
    ) -> Result<Self, sqlx::Error> {
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

    pub async fn create_or_update_existing_user(
        pool: &PgPool,
        claims: &GoogleClaims,
    ) -> Result<Self, sqlx::Error> {
        let existing_user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM "polymarket"."users" WHERE google_id = $1
            "#,
            claims.sub
        )
        .fetch_optional(pool)
        .await?;

        if let Some(user) = existing_user {
            let updated_user = sqlx::query_as!(
                User,
                r#"
                UPDATE "polymarket"."users" SET
                    email = $1,
                    name = $2,
                    avatar = $3,
                    last_login = CURRENT_TIMESTAMP
                WHERE id = $4
                RETURNING *
                "#,
                claims.email,
                claims.name,
                claims.picture,
                user.id
            )
            .fetch_one(pool)
            .await?;

            log_info!("User updated {}", updated_user.id);
            Ok(updated_user)
        } else {
            // Create a new user
            Self::create_new_user(pool, claims).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use std::env;
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;
    use utility_helpers::symmetric::decrypt;

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

        let unique_id = Uuid::new_v5(&Uuid::NAMESPACE_OID, b"test_user");
        let unique_email = format!("test_{}@gmail.com", unique_id);
        let unique_sub = format!("test_google_id_{}", unique_id);

        let google_claims = GoogleClaims {
            sub: unique_sub,
            email: unique_email,
            exp: 60 * 60 * 24 * 3, // 3 days,
            name: "Test User".to_string(),
            picture: "https://example.com/avatar.png".to_string(),
        };

        let user = User::create_new_user(&pool, &google_claims).await.unwrap();

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
