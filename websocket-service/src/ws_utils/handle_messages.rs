use axum::extract::ws::{Message, WebSocket};
use futures::{StreamExt, stream::SplitStream};
use utility_helpers::{log_error, log_info, log_warn};
use uuid::Uuid;

use super::SafeSender;
use crate::{
    AppState,
    ws_utils::{
        ClientMessage, DisconnectReason, MessagePayload, SubscriptionChannel,
        connection_handler::send_message,
    },
};

pub(super) async fn handle_messages(
    rx: &mut SplitStream<WebSocket>,
    tx: &SafeSender,
    client_id: Uuid,
    state: AppState,
) -> DisconnectReason {
    while let Some(msg_res) = rx.next().await {
        match msg_res {
            Ok(msg) => match msg {
                Message::Text(text) => {
                    match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(client_message) => {
                            println!("Received message {:?}", client_message);
                            // process the received message...
                            match client_message.payload {
                                MessagePayload::Subscribe { channel, params } => {
                                    let channel_enum = SubscriptionChannel::from_str(&channel);
                                    if let Some(channel) = channel_enum {
                                        log_info!(
                                            "Client {client_id} subscribed to channel: {channel}, params: {params:?}"
                                        );
                                    } else {
                                        log_warn!(
                                            "Client {client_id} tried to subscribe to an invalid channel: {channel}"
                                        );
                                        let error_msg = format!(
                                            "Invalid channel subscription attempt: {}",
                                            channel
                                        );
                                        if let Err(e) = send_message(tx, error_msg.into()).await {
                                            log_error!("Failed to send error message: {e}");
                                            return DisconnectReason::SendError(e.to_string());
                                        }
                                    }
                                }
                                MessagePayload::Unsubscribe { channel } => {
                                    log_info!(
                                        "Client {client_id} unsubscribed from channel: {channel}"
                                    );
                                }
                            }
                        }
                        Err(_) => {
                            let echo_msg =
                                format!("Invalid message format from client {client_id}: {text}");
                            if let Err(e) = send_message(tx, echo_msg.into()).await {
                                log_error!("Failed to send error message: {e}");
                                return DisconnectReason::SendError(e.to_string());
                            }
                        }
                    }
                }
                Message::Close(frame) => {
                    log_info!("Client sent close frame {client_id}");

                    state
                        .write()
                        .await
                        .process_manager
                        .remove_subscriber_without_channel(client_id);
                    return DisconnectReason::ClientClose(frame);
                }
                Message::Binary(_) => {
                    log_warn!("Received binary message, which is not supported!");
                    return DisconnectReason::ProtocolError(
                        "Binary messages are not supported".to_string(),
                    );
                }
                Message::Ping(_) => {
                    log_info!("Received ping from client {client_id}");
                    if let Err(e) = send_message(tx, Message::Pong(vec![].into())).await {
                        log_error!("Failed to send pong response: {e}");
                        return DisconnectReason::SendError(e.to_string());
                    }
                }
                Message::Pong(_) => {
                    log_info!("Received pong from client {client_id}");
                }
            },
            Err(err) => {
                log_error!("Failed to receive message ");
                return DisconnectReason::ProtocolError(err.to_string());
            }
        }
    }

    DisconnectReason::StreamEnded
}

// Fix this and process_manager_v2 vs process_manager.rs
