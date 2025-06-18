use axum::extract::ws::Message as WsSendMessage;
use prost::Message;
use proto_defs::proto_types::ws_market_price::{WsData, WsParamsPayload};
use utility_helpers::{log_error, log_info, ws::types::ChannelType};
use uuid::Uuid;

use crate::{SafeAppState, utils::send_message};

pub async fn price_poster_handler_bin(
    data: &WsData,
    state: &SafeAppState,
    client_id: &Uuid,
) -> usize {
    let mut served_clients = 0;
    if let Ok(msg_payload) = serde_json::from_str::<WsParamsPayload>(&data.params) {
        // broadcast the message to all clients
        let clients = state.client_manager.write().await;
        let clients = clients.get_clients(&ChannelType::PriceUpdate);
        let data_to_send = msg_payload.encode_to_vec();

        if let Some(clients) = clients {
            for (client_id, client_tx) in clients.iter() {
                if let Err(e) = send_message(
                    client_tx,
                    WsSendMessage::Binary(data_to_send.clone().into()),
                )
                .await
                {
                    log_error!("Failed to send message to {client_id} - {e:#?}");
                } else {
                    served_clients += 1;
                }
            }
        }
    } else {
        log_error!(
            "Failed to parse params from client {client_id}: {}",
            data.params
        );
    }

    log_info!("Served {served_clients} clients");

    served_clients
}
