use crate::log_info;
use deadpool_redis::{Config, Pool, Runtime, redis::AsyncCommands};
use serde::{Serialize, de::DeserializeOwned};
use std::future::Future;

#[derive(Clone, Debug)]
pub struct RedisHelper {
    pool: Pool,
    cache_expiry: u64,
}

impl RedisHelper {
    pub async fn new(url: &str, cache_expiry: u64) -> Result<Self, Box<dyn std::error::Error>> {
        let cfg = Config::from_url(url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        Ok(RedisHelper { pool, cache_expiry })
    }

    pub async fn get_or_set_cache<T, F, Fut>(
        &self,
        key: &str,
        callback: F,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: DeserializeOwned + Serialize,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, Box<dyn std::error::Error>>>,
    {
        let mut conn = self.pool.get().await?;

        if let Ok(data) = conn.get::<_, Vec<u8>>(key).await {
            if !data.is_empty() {
                match rmp_serde::from_slice(&data) {
                    Ok(decoded) => {
                        log_info!("Cache hit for key: {}", key);
                        return Ok(decoded);
                    }
                    Err(e) => {
                        log_info!("Cache corruption detected for key: {}, error: {}", key, e);
                        let _: () = conn.del(key).await.unwrap_or(());
                    }
                }
            }
        }

        let fresh_data = callback().await?;

        match rmp_serde::to_vec_named(&fresh_data) {
            Ok(encoded) => {
                if !encoded.is_empty() {
                    if let Err(e) = conn
                        .set_ex::<&str, Vec<u8>, String>(key, encoded, self.cache_expiry)
                        .await
                    {
                        log_info!("Failed to set cache for key: {}, error: {}", key, e);
                    } else {
                        log_info!("Cache miss, set new value for key: {}", key);
                    }
                }
            }
            Err(e) => {
                log_info!("Failed to serialize data for key: {}, error: {}", key, e);
            }
        }

        Ok(fresh_data)
    }

    pub async fn clear_cache(&self, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.pool.get().await?;
        let _: () = conn.del(key).await?;
        log_info!("Cleared cache for key: {}", key);
        Ok(())
    }

    pub async fn is_cache_valid<T>(&self, key: &str) -> bool
    where
        T: DeserializeOwned,
    {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(_) => return false,
        };

        if let Ok(data) = conn.get::<_, Vec<u8>>(key).await {
            if !data.is_empty() {
                return rmp_serde::from_slice::<T>(&data).is_ok();
            }
        }
        false
    }
}
