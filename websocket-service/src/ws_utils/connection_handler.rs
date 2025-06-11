use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::{Mutex, broadcast};
use utility_helpers::{log_error, log_info, log_warn};
use uuid::Uuid;

use super::SafeSender;
use crate::{
    AppState,
    ws_utils::{
        BroadcastMessage, DisconnectReason, SubscriptionChannel,
        handle_broadcast::handle_broadcast, handle_messages::handle_messages,
    },
};

pub async fn handle_connection(stream: WebSocket, state: AppState) {
    let (tx, mut rx) = stream.split();
    let client_id = Uuid::new_v4();

    log_info!("Client connected {} handling", client_id);
    let tx: SafeSender = Arc::new(Mutex::new(tx));

    let message_handler = {
        let tx = tx.clone();
        let state = state.clone();

        tokio::spawn(async move {
            let disconnect_reason = handle_messages(&mut rx, &tx, client_id, state).await;
            let heart_beat_handler = start_heartbeat(tx.clone(), client_id).await;

            heart_beat_handler.abort();

            cleanup_clients(client_id, disconnect_reason, Some(heart_beat_handler)).await;
        })
    };

    message_handler.await.unwrap_or_else(|e| {
        log_error!("Error in message handler for client {}: {}", client_id, e);
    });

    state
        .read()
        .await
        .process_manager
        .cleanup_client(client_id)
        .await;
}

pub(super) async fn get_client_broadcast_receivers(
    client_id: Uuid,
    state: &AppState,
) -> Vec<(SubscriptionChannel, broadcast::Receiver<BroadcastMessage>)> {
    let state_guard = state.read().await;
    let subscribers = state_guard.process_manager.subscribers.read().await;

    let mut receivers = Vec::new();

    for (channel, channel_subscriber) in subscribers.iter() {
        if let Some(sender) = channel_subscriber.get(&client_id) {
            let receiver = sender.subscribe();
            receivers.push((channel.clone(), receiver));
        }
    }

    receivers
}

pub(super) async fn setup_client_subscriptions(client_id: Uuid, tx: &SafeSender, state: &AppState) {
    let receivers = get_client_broadcast_receivers(client_id, state).await;
    if !receivers.is_empty() {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            handle_broadcast(&tx_clone, client_id, receivers);
        });
    };
}

pub(super) async fn send_message(tx: &SafeSender, message: Message) -> Result<(), axum::Error> {
    tx.lock().await.send(message).await
}

async fn cleanup_clients(
    client_id: Uuid,
    reason: DisconnectReason,
    heart_beat_handler: Option<tokio::task::JoinHandle<()>>,
) {
    log_warn!("Client {client_id} disconnected {:?}, cleaning up", reason);

    if let Some(heart_beat_handler) = heart_beat_handler {
        if !heart_beat_handler.is_finished() {
            heart_beat_handler.abort();
        }
    }

    match reason {
        DisconnectReason::ClientClose(close_frame) => {
            if let Some(frame) = close_frame {
                log_info!("Close code: {}, reason: {}", frame.code, frame.reason);
            }
        }
        DisconnectReason::ProtocolError(error) => {
            log_warn!("Protocol error cleanup for {}: {}", client_id, error);
        }
        DisconnectReason::SendError(error) => {
            log_warn!("Send error cleanup for {}: {}", client_id, error);
        }
        DisconnectReason::StreamEnded => {
            log_info!("Stream ended normally for {}", client_id);
        }
    }
}

async fn start_heartbeat(tx: SafeSender, client_id: Uuid) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

        loop {
            interval.tick().await;

            if let Err(e) = send_message(&tx, Message::Ping(vec![].into())).await {
                log_error!("Heartbeat failed for client {client_id}: {e}");
                break;
            }
        }
    })
}
