use axum::body::Bytes;
use prost::Message;
use proto_defs::proto_types::ws_common_types::{Channel, OperationType, WsMessage};
use utility_helpers::log_info;
use uuid::Uuid;

use crate::{
    SafeAppState,
    utils::{
        client_manager::SpecialKindOfClients,
        message_handlers::channel_handlers::price_posters::price_poster_handler_bin,
    },
};

// binary messages are only sended by poster channels
pub async fn handle_binary_message(message: &Bytes, client_id: &Uuid, state: &SafeAppState) {
    log_info!(
        "Received binary message from client {client_id}: {} bytes",
        message.len()
    );

    let buff: Vec<u8> = message.to_vec();

    let ws_message = WsMessage::decode(buff.as_slice());

    match ws_message {
        Ok(msg) => {
            if let Some(payload) = msg.payload {
                let type_c: OperationType = payload.ops();
                match type_c {
                    OperationType::Post => {
                        if let Some(data) = payload.data {
                            let channel = data.channel();
                            match channel {
                                Channel::Priceposter => {
                                    price_poster_handler_bin(&data, state, client_id).await;
                                }
                                _ => {}
                            }
                        }
                    }
                    OperationType::Handshake => {
                        /*
                         * This is specially used to connect order service to the websocket server and identifies the order-service's (for now) client id
                         *
                         * Example payload
                         * {
                         *  "ops": "Handshake",
                         *  "data": {
                         *     "channel": "OrderService",
                         *     "params": "shared-secret"
                         *  }
                         * }
                         */
                        if let Some(data) = payload.data {
                            let channel = data.channel();
                            match channel {
                                Channel::Orderservice => {
                                    // loading .env file to get the shared secret
                                    dotenv::dotenv().ok();
                                    let shared_secret_env = std::env::var("SHARED_SECRET")
                                        .unwrap_or_else(|_| {
                                            log_info!("SHARED_SECRET not found in .env file");
                                            String::new()
                                        });

                                    let shared_secret = data.params;

                                    if shared_secret != shared_secret_env {
                                        log_info!(
                                            "Handshake failed for OrderService with client ID: {}. Invalid shared secret.",
                                            client_id
                                        );
                                        return;
                                    }

                                    log_info!(
                                        "Handshake successful for OrderService with client ID: {}",
                                        client_id
                                    );
                                    let mut client_manager_guard =
                                        state.client_manager.write().await;
                                    client_manager_guard.set_special_client(
                                        *client_id,
                                        SpecialKindOfClients::OrderService,
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {
                        log_info!(
                            "Unsupported operation type from client {client_id}: {:?}",
                            type_c
                        );
                    }
                }
            }
        }
        Err(e) => {
            log_info!(
                "Failed to decode binary message from client {client_id}: {}",
                e
            );
        }
    }
}
