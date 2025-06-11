use tokio::sync::broadcast;
use utility_helpers::{log_error, log_info};
use uuid::Uuid;

use crate::ws_utils::{
    BroadcastMessage, SafeSender, SubscriptionChannel, connection_handler::send_message,
};

pub fn handle_broadcast(
    tx: &SafeSender,
    client_id: Uuid,
    receivers: Vec<(SubscriptionChannel, broadcast::Receiver<BroadcastMessage>)>,
) {
    log_info!("Starting broadcast handler for client: {}", client_id);

    if receivers.is_empty() {
        log_info!("No receivers to handle for client: {}", client_id);
        return;
    }

    for (channel, mut receiver) in receivers {
        tokio::spawn({
            let tx = tx.clone();

            async move {
                log_info!(
                    "Listening for broadcast on channel {} for client: {}",
                    channel,
                    client_id
                );

                while let Ok(message) = receiver.recv().await {
                    log_info!(
                        "Received broadcast message for client {client_id} channel {}: {:?}",
                        channel,
                        message
                    );

                    let message = serde_json::json!({
                        "type":"broadcast",
                        "channel": message.channel,
                        "data": message.data,
                        "timestamp": message.timestamp
                    });

                    if let Err(e) = send_message(&tx, message.to_string().into()).await {
                        log_error!(
                            "Failed to send broadcast message to client {client_id} on channel {}: {}",
                            channel,
                            e
                        );
                        break;
                    }
                }

                log_info!(
                    "Broadcast handler for client {} on channel {} has ended",
                    client_id,
                    channel
                );
            }
        });
    }
}
