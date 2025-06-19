use std::sync::Arc;

use sqlx::PgPool;
use utility_helpers::{log_info, types::EnvVarConfig};

pub type SafeState = Arc<AppState>;
pub struct AppState {
    pub db_pool: PgPool,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let env_config = EnvVarConfig::new()?;

        let db_pool = PgPool::connect(&env_config.database_url).await?;
        log_info!("Connected to Postgres");

        Ok(AppState { db_pool })
    }
}
