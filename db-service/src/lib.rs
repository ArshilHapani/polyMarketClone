use dotenv::dotenv;
use std::{env, error::Error as StdError};

pub mod pagination;
pub mod schema;
pub mod utils;

pub struct DbService {
    pub pool: sqlx::PgPool,
}

impl DbService {
    pub async fn new() -> Result<Self, Box<dyn StdError>> {
        dotenv().ok();

        let db_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL must be set in the environment variables".to_string())?;

        let pool = sqlx::PgPool::connect(&db_url).await?;

        let db_service = DbService { pool };

        Ok(db_service)
    }
}
