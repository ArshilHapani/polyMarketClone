use async_nats::jetstream;
use db_service::schema::{enums::OrderStatus, orders::Order};
use futures_util::stream::StreamExt;
use state::AppState;
use std::sync::Arc;
use utility_helpers::{log_error, log_info};

use crate::order_book_v2_handler::order_book_v2_handler;

mod order_book_v2;
mod order_book_v2_handler;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app_state = initialize_app().await?;

    log_info!("Connected to NATS JetStream");

    let stream = app_state
        .jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: "ORDERS".to_string(),
            subjects: vec!["orders.>".to_string()],
            ..Default::default()
        })
        .await?;

    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("orders".to_string()),
            ..Default::default()
        })
        .await?;

    let mut messages = consumer.messages().await?;

    while let Some(message) = messages.next().await {
        let message = message?;
        let order_id = String::from_utf8(message.payload.to_vec())
            .map_err(|_| "Failed to convert payload to string".to_string())?;
        log_info!("Received order ID: {}", order_id);
        let _ = order_book_v2_handler(Arc::clone(&app_state), order_id)
            .await
            .map_err(|e| {
                log_error!("Error occur while {e}");
            });

        message
            .ack()
            .await
            .map_err(|_| "Failed to acknowledge message".to_string())?;
    }

    Ok(())
}

async fn initialize_app() -> Result<Arc<AppState>, Box<dyn std::error::Error>> {
    let app_state = Arc::new(AppState::new().await?);

    let open_orders = Order::get_all_open_or_unspecified_orders(&app_state.db_pool).await?;
    {
        let mut global_book = app_state.order_book.write();

        let mut order_ctn = 0;
        // iterate over open orders
        for db_order in open_orders {
            if db_order.status != OrderStatus::OPEN {
                continue;
            }
            let liquidity_b = db_order.liquidity_b.clone();
            let mut order: Order = db_order.into();
            global_book.process_order(&mut order, liquidity_b);
            order_ctn += 1;
        }
        log_info!("Loaded {} open orders into the global book", order_ctn);
    }
    Ok(app_state)
}
