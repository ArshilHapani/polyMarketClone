use std::sync::Arc;

use async_nats::jetstream;
use futures_util::StreamExt;
use utility_helpers::{log_error, log_info};

use crate::{handlers::order_book_handler::order_book_handler, state::AppState};

pub async fn handle_nats_message(
    app_state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
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
        let _ = order_book_handler(Arc::clone(&app_state), order_id)
            .await
            .map_err(|e| {
                log_error!("Error occur while adding order in book {e}");
            });

        message
            .ack()
            .await
            .map_err(|_| "Failed to acknowledge message".to_string())?;
    }

    Ok(())
}
