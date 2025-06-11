use axum::extract::ws::{Message, WebSocket};
use futures::{StreamExt, stream::SplitStream};
use tokio::sync::broadcast;
use utility_helpers::{log_error, log_info, log_warn};
use uuid::Uuid;

use super::SafeSender;
use crate::{
    AppState,
    ws_utils::{
        BroadcastMessage, ClientMessage, DisconnectReason, MessagePayload, SubscriptionChannel,
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
                            let process_manager = &state.read().await.process_manager;

                            match &client_message.payload {
                                MessagePayload::Subscribe { channel, params } => {
                                    match process_manager
                                        .subscribe_client_with_receiver(
                                            client_id,
                                            channel,
                                            params.clone(),
                                        )
                                        .await
                                    {
                                        Ok(Some((response, receiver))) => {
                                            let success_message = serde_json::json!({
                                                "status":"success",
                                                "message": response
                                            });
                                            if let Err(e) =
                                                send_message(tx, success_message.to_string().into())
                                                    .await
                                            {
                                                log_error!("Failed to send success message: {e}");
                                                return DisconnectReason::SendError(e.to_string());
                                            }

                                            let channel_type =
                                                SubscriptionChannel::from_str(channel).unwrap();
                                            let tx_clone = tx.clone();
                                            tokio::spawn(async move {
                                                handle_single_channel_broadcast(
                                                    &tx_clone,
                                                    client_id,
                                                    channel_type,
                                                    receiver,
                                                )
                                                .await;
                                            });
                                        }
                                        Ok(None) => {
                                            log_info!(
                                                "Subscription processed but no receiver returned"
                                            );
                                        }
                                        Err(e) => {
                                            log_error!("Failed to subscribe client: {e}");
                                            let error_message = serde_json::json!({
                                                "status": "error",
                                                "message": e.to_string()
                                            });
                                            if let Err(e) =
                                                send_message(tx, error_message.to_string().into())
                                                    .await
                                            {
                                                log_error!("Failed to send error message: {e}");
                                                return DisconnectReason::SendError(e.to_string());
                                            }
                                        }
                                    }
                                }
                                MessagePayload::Unsubscribe { channel } => {
                                    match process_manager
                                        .unsubscribe_client(client_id, channel)
                                        .await
                                    {
                                        Ok(Some(response)) => {
                                            let success_msg = serde_json::json!({
                                                "status": "success",
                                                "message": response
                                            });
                                            if let Err(e) =
                                                send_message(tx, success_msg.to_string().into())
                                                    .await
                                            {
                                                log_error!("Failed to send success message: {e}");
                                                return DisconnectReason::SendError(e.to_string());
                                            }
                                        }
                                        Ok(None) => {
                                            log_info!(
                                                "Unsubscription processed but no response returned"
                                            );
                                        }
                                        Err(e) => {
                                            log_error!("Failed to unsubscribe client: {e}");
                                            let error_msg = serde_json::json!({
                                                "status": "error",
                                                "message": e.to_string()
                                            });
                                            if let Err(e) =
                                                send_message(tx, error_msg.to_string().into()).await
                                            {
                                                log_error!("Failed to send error message: {e}");
                                                return DisconnectReason::SendError(e.to_string());
                                            }
                                        }
                                    }
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
                        .read()
                        .await
                        .process_manager
                        .cleanup_client(client_id)
                        .await;

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

    state
        .read()
        .await
        .process_manager
        .cleanup_client(client_id)
        .await;

    DisconnectReason::StreamEnded
}

async fn handle_single_channel_broadcast(
    tx: &SafeSender,
    client_id: Uuid,
    channel: SubscriptionChannel,
    mut receiver: broadcast::Receiver<BroadcastMessage>,
) {
    log_info!("Starting broadcast handler for client: {}", client_id);

    while let Ok(msg) = receiver.recv().await {
        let message = serde_json::json!({
            "type": "broadcast",
            "channel": msg.channel,
            "data": msg.data,
            "timestamp": msg.timestamp
        });

        if let Err(e) = send_message(tx, message.to_string().into()).await {
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
