use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::Mutex;
use utility_helpers::{log_error, log_info, log_warn};
use uuid::Uuid;

use super::SafeSender;
use crate::{
    AppState,
    ws_utils::{DisconnectReason, handle_messages::handle_messages},
};

pub async fn handle_connection(stream: WebSocket, state: AppState) {
    let (tx, mut rx) = stream.split();
    let client_id = Uuid::new_v4();

    log_info!("Client connected {} handling", client_id);
    let tx: SafeSender = Arc::new(Mutex::new(tx));

    let heart_beat_handler = start_heartbeat(tx.clone(), client_id).await;
    let disconnect_reason = handle_messages(&mut rx, &tx, client_id, state.clone()).await;

    heart_beat_handler.abort();

    cleanup_clients(
        state,
        client_id,
        disconnect_reason,
        Some(heart_beat_handler),
    )
    .await;
}

pub(super) async fn send_message(tx: &SafeSender, message: Message) -> Result<(), axum::Error> {
    tx.lock().await.send(message).await
}

async fn cleanup_clients(
    _state: AppState,
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
