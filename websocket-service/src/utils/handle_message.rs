use axum::extract::ws::{Message, WebSocket};
use futures::{StreamExt, stream::SplitStream};
use serde_json::json;
use utility_helpers::{log_error, log_info, types::ChannelType};
use uuid::Uuid;

use crate::{
    SafeAppState,
    utils::{
        ClientMessage, MessagePayload, SafeSender,
        process_channel_request::process_channel_request, send_message,
    },
};

pub async fn handle_message(
    rx: &mut SplitStream<WebSocket>,
    tx: &SafeSender,
    client_id: Uuid,
    state: &SafeAppState,
) {
    while let Some(message) = rx.next().await {
        match message {
            Ok(message) => {
                match message {
                    Message::Text(text) => match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(client_message) => {
                            match client_message.payload {
                                MessagePayload::Subscribe { channel, params } => {
                                    log_info!(
                                        "Client {client_id} subscribed to channel: {channel}, params: {params:?}"
                                    );

                                    let deserialized_channel =
                                        ChannelType::from_str_serde(&channel);

                                    let channel_type = match deserialized_channel {
                                        Ok(channel_type) => channel_type,
                                        Err(_) => {
                                            log_error!(
                                                "Invalid channel type from client {client_id}: {channel}"
                                            );
                                            if let Err(e) = send_message(
                                                tx,
                                                Message::Text("Invalid channel".into()),
                                            )
                                            .await
                                            {
                                                log_error!(
                                                    "Failed to send error response to client {client_id}: {e}"
                                                );
                                            }
                                            return;
                                        }
                                    };

                                    let mut channel_manager_guard =
                                        state.client_manager.write().await;

                                    channel_manager_guard.add_client(
                                        channel_type.clone(),
                                        client_id,
                                        tx.clone(),
                                        params.clone(),
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
                                    let deserialized_channel =
                                        ChannelType::from_str_serde(&channel);
                                    let channel_type = match deserialized_channel {
                                        Ok(channel_type) => channel_type,
                                        Err(_) => {
                                            log_error!(
                                                "Invalid channel type from client {client_id}: {channel}"
                                            );
                                            if let Err(e) = send_message(
                                                tx,
                                                Message::Text("Invalid channel".into()),
                                            )
                                            .await
                                            {
                                                log_error!(
                                                    "Failed to send error response to client {client_id}: {e}"
                                                );
                                            }
                                            return;
                                        }
                                    };

                                    // processing channel request
                                    let served_clients = process_channel_request(
                                        &channel_type,
                                        client_id,
                                        &data,
                                        state,
                                    )
                                    .await;

                                    log_info!(
                                        "Processed post request from client {client_id} on channel: {channel}"
                                    );

                                    let message = format!(
                                        "Data posted to channel {}. Served {} clients.",
                                        channel, served_clients
                                    );
                                    if let Err(e) =
                                        send_message(tx, Message::Text(message.into())).await
                                    {
                                        log_error!(
                                            "Failed to send post confirmation to client {client_id}: {e}"
                                        );
                                    }
                                }

                                MessagePayload::Unsubscribe { channel } => {
                                    log_info!(
                                        "Client {client_id} unsubscribed from channel: {channel}"
                                    );

                                    let deserialized_channel =
                                        ChannelType::from_str_serde(&channel);

                                    let channel_type = match deserialized_channel {
                                        Ok(channel_type) => channel_type,
                                        Err(_) => {
                                            log_error!(
                                                "Invalid channel type from client {client_id}: {channel}"
                                            );
                                            if let Err(e) = send_message(
                                                tx,
                                                Message::Text("Invalid channel".into()),
                                            )
                                            .await
                                            {
                                                log_error!(
                                                    "Failed to send error response to client {client_id}: {e}"
                                                );
                                            }
                                            return;
                                        }
                                    };

                                    let mut channel_manager_guard =
                                        state.client_manager.write().await;
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
                            log_error!(
                                "Failed to parse ClientMessage from client {client_id}: {e}"
                            );
                            if let Err(e) =
                                send_message(tx, Message::Text("Invalid message format".into()))
                                    .await
                            {
                                log_error!(
                                    "Failed to send error response to client {client_id}: {e}"
                                );
                            }
                        }
                    },
                    Message::Pong(_) => {
                        log_info!("Received Pong from client {client_id}");
                    }
                    Message::Ping(_) => {
                        log_info!("Received Ping from client {client_id}");
                        // Optionally, you can respond with a Pong
                        if let Err(e) = send_message(tx, Message::Pong(vec![].into())).await {
                            log_error!("Failed to send Pong to client {client_id}: {e}");
                        }
                    }
                    _ => {
                        log_info!(
                            "Received unsupported message type from client {client_id}: {message:?}"
                        );
                    }
                }
            }
            Err(e) => {
                log_error!("Error receiving message from client {client_id}: {e}");
            }
        }
    }
}
