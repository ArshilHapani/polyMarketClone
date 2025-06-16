use axum::extract::ws::{Message, Utf8Bytes};
use serde_json::json;
use utility_helpers::{
    log_error, log_info,
    ws::types::{ChannelType, ClientMessage, MessagePayload},
};
use uuid::Uuid;

use crate::{
    SafeAppState,
    utils::{SafeSender, process_channel_request::process_channel_request, send_message},
};

pub async fn handle_text_message(
    message: &Utf8Bytes,
    client_id: &Uuid,
    tx: &SafeSender,
    state: &SafeAppState,
) {
    match serde_json::from_str::<ClientMessage>(message) {
        Ok(client_message) => {
            match client_message.payload {
                MessagePayload::Subscribe { channel, params } => {
                    log_info!(
                        "Client {client_id} subscribed to channel: {channel}, params: {params:?}"
                    );

                    let deserialized_channel = ChannelType::from_str_serde(&channel);

                    let channel_type = match deserialized_channel {
                        Ok(channel_type) => channel_type,
                        Err(_) => {
                            log_error!("Invalid channel type from client {client_id}: {channel}");
                            if let Err(e) =
                                send_message(tx, Message::Text("Invalid channel".into())).await
                            {
                                log_error!(
                                    "Failed to send error response to client {client_id}: {e}"
                                );
                            }
                            return;
                        }
                    };

                    let mut channel_manager_guard = state.client_manager.write().await;

                    let params_value = serde_json::to_value(&params).unwrap_or_else(|_| {
                        log_error!("Failed to serialize params for client {client_id}");
                        return serde_json::Value::Null;
                    });

                    channel_manager_guard.add_client(
                        channel_type.clone(),
                        *client_id,
                        tx.clone(),
                        params_value,
                    );

                    let message = json!({
                        "type": "subscribed",
                        "channel": channel,
                        "params": params
                    })
                    .to_string();
                    if let Err(e) = send_message(tx, message.into()).await {
                        log_error!(
                            "Failed to send subscription confirmation to client {client_id}: {e}"
                        );
                    }
                }

                MessagePayload::Post { channel, data } => {
                    log_info!(
                        "Client {client_id} posted data to channel: {channel}, data: {data:?}"
                    );
                    let deserialized_channel = ChannelType::from_str_serde(&channel);
                    let channel_type = match deserialized_channel {
                        Ok(channel_type) => channel_type,
                        Err(_) => {
                            log_error!("Invalid channel type from client {client_id}: {channel}");
                            if let Err(e) =
                                send_message(tx, Message::Text("Invalid channel".into())).await
                            {
                                log_error!(
                                    "Failed to send error response to client {client_id}: {e}"
                                );
                            }
                            return;
                        }
                    };

                    // processing channel request
                    let served_clients =
                        process_channel_request(&channel_type, *client_id, &data, state).await;

                    log_info!(
                        "Processed post request from client {client_id} on channel: {channel}"
                    );

                    let message = format!(
                        "Data posted to channel {}. Served {} clients.",
                        channel, served_clients
                    );
                    if let Err(e) = send_message(tx, Message::Text(message.into())).await {
                        log_error!("Failed to send post confirmation to client {client_id}: {e}");
                    }
                }

                MessagePayload::Unsubscribe { channel } => {
                    log_info!("Client {client_id} unsubscribed from channel: {channel}");

                    let deserialized_channel = ChannelType::from_str_serde(&channel);

                    let channel_type = match deserialized_channel {
                        Ok(channel_type) => channel_type,
                        Err(_) => {
                            log_error!("Invalid channel type from client {client_id}: {channel}");
                            if let Err(e) =
                                send_message(tx, Message::Text("Invalid channel".into())).await
                            {
                                log_error!(
                                    "Failed to send error response to client {client_id}: {e}"
                                );
                            }
                            return;
                        }
                    };

                    let mut channel_manager_guard = state.client_manager.write().await;
                    channel_manager_guard.remove_client(&channel_type, &client_id);
                    let message = json!({
                        "type": "unsubscribed",
                        "channel": channel
                    })
                    .to_string();
                    if let Err(e) = send_message(tx, message.into()).await {
                        log_error!(
                            "Failed to send unsubscription confirmation to client {client_id}: {e}"
                        );
                    }
                }
            }
        }
        Err(e) => {
            log_error!("Failed to parse ClientMessage from client {client_id}: {e}");
            if let Err(e) = send_message(tx, Message::Text("Invalid message format".into())).await {
                log_error!("Failed to send error response to client {client_id}: {e}");
            }
        }
    }
}
