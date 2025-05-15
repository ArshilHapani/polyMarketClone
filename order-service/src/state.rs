use std::sync::Arc;

use async_nats::{
    connect,
    jetstream::{self, Context},
};
use parking_lot::RwLock;
use utility_helpers::types::EnvVarConfig;

use crate::order_book::GlobalOrderBook;

pub struct AppState {
    pub jetstream: Context,
    pub db_pool: sqlx::PgPool,
    pub order_book: Arc<RwLock<GlobalOrderBook>>,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        let env_var_config = EnvVarConfig::new()?;

        let nc = connect(&env_var_config.nc_url).await?;
        let js = jetstream::new(nc);
        let db_pool = sqlx::PgPool::connect(&env_var_config.database_url).await?;
        let order_book = Arc::new(RwLock::new(GlobalOrderBook::new()));

        Ok(AppState {
            jetstream: js,
            db_pool,
            order_book,
        })
    }
}
