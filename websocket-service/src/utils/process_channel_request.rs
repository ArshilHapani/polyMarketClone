use utility_helpers::log_error;
use uuid::Uuid;

use crate::{
    SafeAppState,
    utils::{ChannelType, send_message},
};

pub async fn process_channel_request(
    channel: &ChannelType,
    client_id_: Uuid,
    params: &serde_json::Value,
    state: &SafeAppState,
) -> usize {
    let mut served_clients = 0;
    match channel {
        ChannelType::PricePoster => {
            // send price update from params to all subscribers of PriceUpdate channel
            let channel_manager_guard = state.client_manager.write().await;

            if let Some(clients) = channel_manager_guard.get_clients(&ChannelType::PriceUpdate) {
                for (client_id, (tx, _)) in clients.iter() {
                    if client_id_ == *client_id {
                        continue;
                    }
                    if let Err(e) = send_message(tx, params.to_string().into()).await {
                        log_error!("Failed to send price update to client {client_id}: {e}");
                        continue;
                    }
                    served_clients += 1;
                }
            } else {
                log_error!("No subscribers found for PriceUpdate channel");
            }
        }
        _ => {
            log_error!("Unsupported channel type: {:?}", channel);
        }
    };

    served_clients
}
