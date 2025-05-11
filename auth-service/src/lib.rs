pub mod symmetric;
pub mod token_services;
pub mod types;

use deadpool_redis::{Config, Pool, redis::AsyncCommands};
use dotenv::dotenv;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use reqwest::Client;
use std::{collections::HashMap, env, error::Error as StdError};
use token_services::Claims;

use types::{GoogleClaims, GoogleClaimsError};

pub struct AuthService {
    pub redis_pool: Pool,
    pub client: Client,
}

impl AuthService {
    pub fn new() -> Result<Self, Box<dyn StdError>> {
        dotenv().ok();

        let redis_url = env::var("REDIS_URL")
            .map_err(|_| "REDIS_URL must be set in the environment variables".to_string())?;
        let cfg = Config::from_url(&redis_url);

        let pool = cfg.create_pool(None)?;
        let client = Client::new();

        let auth_service = AuthService {
            redis_pool: pool,
            client,
        };

        Ok(auth_service)
    }

    pub fn get_claims(data: String, exp: usize) -> Claims {
        Claims::new(data, exp)
    }

    pub async fn generate_refresh_token(
        &self,
        claims: &Claims,
    ) -> Result<String, Box<dyn StdError>> {
        let token = claims.new_token()?;
        let mut conn = self.redis_pool.get().await?;
        let expiry = 60 * 60 * 24 * 3; // 3 days

        let _: () = conn.set_ex(&claims.sub, &token, expiry).await?;
        Ok(token)
    }

    pub async fn verify_refresh_token(&self, token: &String) -> Result<Claims, Box<dyn StdError>> {
        let mut conn = self.redis_pool.get().await?;

        let claims = Claims::verify_token(token.as_str())?;

        let stored_token: Option<String> = conn.get(&claims.sub).await?;

        if stored_token.is_none() {
            return Err("Token not found in Redis".into());
        }
        if stored_token.unwrap() == *token {
            Ok(claims)
        } else {
            Err("Token mismatch".into())
        }
    }

    pub async fn get_google_claims(
        &self,
        id_token: &String,
    ) -> Result<GoogleClaims, GoogleClaimsError> {
        let client = self.client.clone();

        let header = decode_header(id_token).map_err(|_| GoogleClaimsError::InvalidTokenId)?;
        let kid = header.kid.ok_or(GoogleClaimsError::MissingKid)?;

        let keys: Result<HashMap<String, DecodingKey>, GoogleClaimsError> = {
            let res = client
                .get("https://www.googleapis.com/oauth2/v3/certs")
                .send()
                .await
                .map_err(|_| GoogleClaimsError::FailedToGetKeyFromGoogle)?;

            let jwks: serde_json::Value = res
                .json()
                .await
                .map_err(|_| GoogleClaimsError::InvalidResponseTypeFromGoogle)?;

            jwks["keys"]
                .as_array()
                .ok_or(GoogleClaimsError::InvalidResponseTypeFromGoogle)?
                .iter()
                .map(|k| {
                    let kid = k["kid"]
                        .as_str()
                        .ok_or(GoogleClaimsError::InvalidKeyComponentFromGoogle)?;
                    let n = k["n"]
                        .as_str()
                        .ok_or(GoogleClaimsError::InvalidKeyComponentFromGoogle)?;
                    let e = k["e"]
                        .as_str()
                        .ok_or(GoogleClaimsError::InvalidKeyComponentFromGoogle)?;

                    Ok((
                        kid.to_string(),
                        DecodingKey::from_rsa_components(n, e)
                            .map_err(|_| GoogleClaimsError::FailedToDecodeRsaComponents)?,
                    ))
                })
                .collect()
        };

        let keys = keys?;

        let key = keys.get(&kid).ok_or(GoogleClaimsError::KeyNotFound)?;

        let token_data = decode::<GoogleClaims>(&id_token, key, &Validation::new(Algorithm::RS256))
            .map_err(|_| GoogleClaimsError::InvalidTokenId)?;

        let claims = token_data.claims;
        if claims.exp < chrono::Utc::now().timestamp() as usize {
            return Err(GoogleClaimsError::ExpiredToken);
        }
        Ok(claims)
    }
}
