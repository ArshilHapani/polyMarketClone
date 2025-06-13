use std::sync::Arc;

use async_nats::connect;
use parking_lot::RwLock;
use rdkafka::{ClientConfig, producer::FutureProducer};
use tokio::{net::TcpStream, sync::RwLock as AsyncRwLock};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::client::IntoClientRequest,
};
use utility_helpers::types::EnvVarConfig;

use crate::order_book::global_book::GlobalMarketBook;

pub struct AppState {
    pub db_pool: sqlx::PgPool,
    // prefering RwLock rather than tokio's rwLock because the operations on orderbook are not async
    pub order_book: Arc<RwLock<GlobalMarketBook>>,
    pub jetstream: AsyncRwLock<async_nats::jetstream::Context>,
    pub producer: AsyncRwLock<FutureProducer>,
    pub websocket_stream: AsyncRwLock<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        let env_var_config = EnvVarConfig::new()?;

        let nc = connect(&env_var_config.nc_url)
            .await
            .expect("Failed to connect to NATS server");
        let jetstream = async_nats::jetstream::new(nc);
        let db_pool = sqlx::PgPool::connect(&env_var_config.database_url)
            .await
            .expect("Failed to connect to the database");
        let order_book = Arc::new(RwLock::new(GlobalMarketBook::new()));
        let producer = ClientConfig::new()
            .set("bootstrap.servers", &env_var_config.kafka_url)
            .create::<FutureProducer>()
            .expect("Failed to create Kafka producer");

        let websocket_req = format!("{}/ws", env_var_config.websocket_url)
            .into_client_request()
            .expect("Failed to create WebSocket request");
        let (stream, _) = connect_async(websocket_req)
            .await
            .expect("Failed to connect to WebSocket server");

        Ok(AppState {
            db_pool,
            order_book,
            jetstream: AsyncRwLock::new(jetstream),
            producer: AsyncRwLock::new(producer),
            websocket_stream: AsyncRwLock::new(stream),
        })
    }
}
