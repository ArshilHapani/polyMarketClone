use dotenv::dotenv;
use std::error::Error as StdError;
use utility_helpers::types::EnvVarConfig;

pub mod pagination;
pub mod procedures;
pub mod schema;
pub mod utils;

pub struct DbService {
    pub pool: sqlx::PgPool,
    pub env_var_config: EnvVarConfig,
}

impl DbService {
    pub async fn new() -> Result<Self, Box<dyn StdError>> {
        dotenv().ok();

        let env_var_config = EnvVarConfig::new()?;

        let pool = sqlx::PgPool::connect(&env_var_config.database_url).await?;

        let db_service = DbService {
            pool,
            env_var_config,
        };

        Ok(db_service)
    }
}
