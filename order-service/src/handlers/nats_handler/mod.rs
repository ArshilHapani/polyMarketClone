use std::sync::Arc;

use async_nats::jetstream;
use futures_util::StreamExt;
use utility_helpers::{log_error, log_info, nats_helper::NatsSubjects};

use crate::{
    handlers::nats_handler::{
        cancel_order_handler::cancel_order_handler, create_order_handler::create_order_handler,
    },
    state::AppState,
};

pub mod cancel_order_handler;
pub mod create_order_handler;

pub async fn handle_nats_message(
    app_state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    let stream_guard = app_state.jetstream.clone();
    let stream = stream_guard
        .get_or_create_stream(jetstream::stream::Config {
            // these `ORDER` name does not indicate the operations on orders, instead it indicates that the streams is used by order-service microservice, so don't confuse it with the order name and same for it's topics, all topics are prefixed with `order.`
            name: "ORDER".to_string(),
            subjects: vec!["order.>".to_string()],
            ..Default::default()
        })
        .await?;

    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("order_os".to_string()),
            ..Default::default()
        })
        .await?;

    let mut messages = consumer.messages().await?;

    while let Some(Ok(message)) = messages.next().await {
        let subject = message.subject.clone();
        let subject_str = subject.as_str();
        let subject = NatsSubjects::from_string(subject_str)
            .ok_or_else(|| format!("Invalid subject: {}", subject))?;

        match subject {
            NatsSubjects::OrderCreate => {
                let order_id = String::from_utf8(message.payload.to_vec())
                    .map_err(|_| "Failed to convert payload to string".to_string())?;
                log_info!("Received order ID: {}", order_id);
                let _ = create_order_handler(Arc::clone(&app_state), order_id)
                    .await
                    .map_err(|e| {
                        log_error!("Error occur while adding order in book {e}");
                    });
            }
            NatsSubjects::OrderCancel => {
                let order_id = String::from_utf8(message.payload.to_vec())
                    .map_err(|_| "Failed to convert payload to string".to_string())?;
                let _ = cancel_order_handler(app_state.clone(), order_id)
                    .await
                    .map_err(|e| {
                        log_error!("Error occur while cancelling order {e}");
                    });
            }
            NatsSubjects::MarketBookUpdate(_) => {}
        }

        // sending ack in either case...
        message
            .ack()
            .await
            .map_err(|_| "Failed to acknowledge message".to_string())?;
    }

    Ok(())
}
