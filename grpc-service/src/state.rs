use std::sync::Arc;

use sqlx::PgPool;
use utility_helpers::{log_info, redis::RedisHelper, types::EnvVarConfig};

pub type SafeState = Arc<AppState>;
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_helper: RedisHelper,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let env_config = EnvVarConfig::new()?;
        let redis_helper = RedisHelper::new(&env_config.redis_url, 60).await?; // 60 seconds TTL for Redis keys

        let db_pool = PgPool::connect(&env_config.database_url).await?;
        log_info!("Connected to Postgres");

        Ok(AppState {
            db_pool,
            redis_helper,
        })
    }
}
