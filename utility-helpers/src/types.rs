use serde::{Deserialize, Serialize};
use std::env::var;

#[derive(Deserialize, Serialize, Debug)]
pub struct GoogleClaims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub picture: String,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct EnvVarConfig {
    pub jwt_secret: String,
    pub secret_key: String,
    pub redis_url: String,
    pub database_url: String,
    pub google_client_id: String,
    pub nc_url: String,
    pub influxdb_url: String,
    pub kafka_url: String,
    pub websocket_url: String,
}

impl EnvVarConfig {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        let jwt_secret =
            var("JWT_SECRET").map_err(|_| "JWT_SECRET environment variable not set".to_string())?;

        let secret_key =
            var("SECRET_KEY").map_err(|_| "SECRET_KEY environment variable not set".to_string())?;
        let redis_url =
            var("REDIS_URL").map_err(|_| "REDIS_URL environment variable not set".to_string())?;
        let database_url = var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable not set".to_string())?;
        let google_client_id = var("GOOGLE_CLIENT_ID")
            .map_err(|_| "GOOGLE_CLIENT_ID environment variable not set".to_string())?;
        let nc_url =
            var("NC_URL").map_err(|_| "NC_URL environment variable not set".to_string())?;

        let influxdb_url = var("INFLUXDB_URL")
            .map_err(|_| "INFLUXDB_URL environment variable not set".to_string())?;

        let kafka_url =
            var("KAFKA_URL").map_err(|_| "KAFKA_URL environment variable not set".to_string())?;

        let websocket_url = var("WS_SERVER_URL")
            .map_err(|_| "WS_SERVER_URL environment variable not set".to_string())?;

        Ok(EnvVarConfig {
            jwt_secret,
            secret_key,
            redis_url,
            database_url,
            google_client_id,
            nc_url,
            influxdb_url,
            kafka_url,
            websocket_url,
        })
    }
}
