use std::error::Error as StdError;

use async_nats::{
    connect,
    jetstream::{self, Context},
};
use auth_service::AuthService;
use utility_helpers::types::EnvVarConfig;

use crate::bloom_f::BloomFilterWrapper;

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: sqlx::PgPool,
    pub auth_service: AuthService,
    pub jetstream: Context,
    pub bloom_filter: BloomFilterWrapper, // already thread safe
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn StdError>> {
        dotenv::dotenv().ok();

        let env_var_config = EnvVarConfig::new()?;

        let ns = connect(&env_var_config.nc_url).await?;
        let jetstream = jetstream::new(ns);

        let pg_pool = sqlx::PgPool::connect(&env_var_config.database_url).await?;
        let auth_service = AuthService::new(pg_pool.clone())?;

        let bloom_filter = BloomFilterWrapper::new(&pg_pool).await?;

        let state = AppState {
            pg_pool,
            auth_service,
            jetstream,
            bloom_filter,
        };

        Ok(state)
    }
}
