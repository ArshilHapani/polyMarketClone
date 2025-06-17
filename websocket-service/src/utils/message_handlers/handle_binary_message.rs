use axum::body::Bytes;
use prost::Message;
use proto_defs::proto_types::ws_market_price::WsMessage;
use utility_helpers::log_info;
use uuid::Uuid;

use crate::{SafeAppState, utils::SafeSender};

pub async fn handle_binary_message(
    message: &Bytes,
    client_id: &Uuid,
    tx: &SafeSender,
    state: &SafeAppState,
) {
    log_info!(
        "Received binary message from client {client_id}: {} bytes",
        message.len()
    );

    let buff: Vec<u8> = message.to_vec();

    let ws_message = WsMessage::decode(buff.as_slice());

    match ws_message {
        Ok(msg) => {
            log_info!("Decoded message from client {client_id}: {:?}", msg);
        }
        Err(e) => {
            log_info!(
                "Failed to decode binary message from client {client_id}: {}",
                e
            );
        }
    }
}
