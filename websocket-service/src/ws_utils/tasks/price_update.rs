use std::sync::Arc;

use tokio::sync::mpsc;
use utility_helpers::{log_error, log_info};
use uuid::Uuid;

use crate::ws_utils::{
    SafeSender, SubscriptionChannel, connection_handler::send_message,
    process_manager_v2::ProcessMessage,
};

pub async fn price_update_task(
    tx: mpsc::Sender<ProcessMessage>,
    payload: serde_json::Value,
    subscribers: Arc<Vec<SafeSender>>,
) {
    log_info!("Starting price update task with payload: {:?}", payload);

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
    let mut iteration = 0;
    loop {
        iteration += 1;
        interval.tick().await;

        // Simulate a price update with a random value
        // let price_update = serde_json::json!({
        //     "symbol": "BTCUSD",
        //     "price": 10000 + iteration * 100, // Incrementing price for demonstration
        // });

        // let message = ProcessMessage {
        //     channel: SubscriptionChannel::PriceUpdates(Uuid::new_v4()),
        //     data: price_update,
        // };

        // if tx.send(message).await.is_err() {
        //     log_error!("Failed to send price update message");
        //     break;
        // };
        let subscribers = subscribers.iter().cloned().collect::<Vec<_>>();
        log_info!("Sending price update to {} subscribers", subscribers.len());
        for client in &subscribers {
            if let Err(e) = send_message(client, format!("{}", 1000 + iteration * 100).into()).await
            {
                log_error!("Failed to send price update to client: {}", e);
            } else {
                log_info!("Sent price update to client");
            }
        }
    }
}
