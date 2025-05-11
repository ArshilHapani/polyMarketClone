use std::error::Error as StdError;

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: sqlx::PgPool,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn StdError>> {
        dotenv::dotenv().ok();
        let database_url =
            std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL not set in .env file")?;

        let pg_pool = sqlx::PgPool::connect(&database_url).await?;
        let state = AppState { pg_pool };

        Ok(state)
    }
}
