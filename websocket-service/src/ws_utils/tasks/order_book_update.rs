use serde_json::json;
use tokio::sync::mpsc;
use utility_helpers::{log_error, log_info};
use uuid::Uuid;

use crate::ws_utils::{SubscriptionChannel, process_manager_v2::ProcessMessage};

pub async fn order_book_update_task(tx: mpsc::Sender<ProcessMessage>, payload: serde_json::Value) {
    log_info!("Starting order book update task with {:?}", payload);

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

    loop {
        interval.tick().await;

        let data = json!({
            "date": chrono::Utc::now().to_rfc3339(),
            "payload": payload.clone(),
        });

        let message = ProcessMessage {
            channel: SubscriptionChannel::OrderBookUpdate(Uuid::new_v4()),
            data,
        };

        if tx.send(message).await.is_err() {
            log_error!("Failed to send order book update message");
            break;
        }
    }
}
