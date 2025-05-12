use std::error::Error as StdError;

use auth_service::AuthService;

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: sqlx::PgPool,
    pub auth_service: AuthService,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn StdError>> {
        dotenv::dotenv().ok();
        let database_url =
            std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL not set in .env file")?;
        let google_client_id = std::env::var("GOOGLE_CLIENT_ID")
            .map_err(|_| "GOOGLE_CLIENT_ID not set in .env file")?;
        let jwt_secret =
            std::env::var("JWT_SECRET").map_err(|_| "JWT_SECRET not set in .env file")?;

        let pg_pool = sqlx::PgPool::connect(&database_url).await?;
        let auth_service = AuthService::new(google_client_id, jwt_secret, pg_pool.clone())?;

        let state = AppState {
            pg_pool,
            auth_service,
        };

        Ok(state)
    }
}
