use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::StreamExt;
use tokio::sync::Mutex;
use utility_helpers::{log_error, log_info};
use uuid::Uuid;

use crate::{
    SafeAppState,
    utils::{SafeSender, handle_message::handle_message, send_message},
};

pub async fn handle_connection(stream: WebSocket, state: SafeAppState) {
    let (tx, mut rx) = stream.split();

    let tx = Arc::new(Mutex::new(tx));
    let client_id = Uuid::new_v4();
    log_info!("New client connected: {client_id}");

    let heart_beat_handler = start_heartbeat(tx.clone(), client_id).await;
    handle_message(&mut rx, &tx, client_id, &state).await;

    // cleanup
    log_info!("Client {client_id} disconnected, cleaning up resources");
    let mut channel_manager_guard = state.client_manager.write().await;
    channel_manager_guard.cleanup();
    heart_beat_handler.abort();
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
