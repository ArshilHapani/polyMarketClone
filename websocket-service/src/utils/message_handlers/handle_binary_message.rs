use axum::body::Bytes;
use prost::Message;
use proto_defs::proto_types::ws_market_price::{Channel, OperationType, WsMessage};
use utility_helpers::log_info;
use uuid::Uuid;

use crate::{
    SafeAppState,
    utils::message_handlers::channel_handlers::price_posters::price_poster_handler_bin,
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
