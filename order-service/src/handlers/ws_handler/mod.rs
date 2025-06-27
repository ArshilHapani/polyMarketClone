use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use utility_helpers::{log_error, log_info, log_warn, ws::types::ClientMessage};

use crate::{
    handlers::ws_handler::handle_client_messages::handle_client_messages, state::AppState,
};

mod handle_client_messages;

pub async fn handle_ws_messages(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let mut ws_stream_guard = state.websocket_stream.write().await;
    let (mut tx, mut rx) = StreamExt::split(&mut *ws_stream_guard);

    while let Some(data) = rx.next().await {
        match data {
            Ok(data) => match data {
                Message::Text(text) => {
                    log_info!("Received text message: {}", text);
                    let parsed_message: Result<ClientMessage, _> = serde_json::from_str(&text);
                    match parsed_message {
                        Ok(client_message) => {
                            log_info!("Parsed client message: {:?}", client_message);
                            handle_client_messages(&state, &client_message).await;
                        }
                        Err(e) => {
                            log_error!("Failed to parse client message: {}", e);
                            continue;
                        }
                    }
                }
                Message::Ping(_) => {
                    log_info!("Received Ping from server");
                    if let Err(e) = tx.send(Message::Pong(vec![].into())).await {
                        log_error!("Failed to send Pong to server : {e}");
                    }
                }

                _ => {
                    log_warn!("Received non-text message: {:?}", data);
                }
            },
            Err(e) => {
                log_error!("WebSocket error: {}", e);
            }
        }
    }
    Ok(())
}
