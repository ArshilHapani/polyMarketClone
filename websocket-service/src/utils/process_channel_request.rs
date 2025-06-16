use serde::{Deserialize, Serialize};
use utility_helpers::{log_error, ws::types::ChannelType};
use uuid::Uuid;

use crate::{SafeAppState, utils::send_message};

pub async fn process_channel_request<T>(
    channel: &ChannelType,
    client_id_: Uuid,
    params: &T,
    state: &SafeAppState,
) -> usize
where
    T: Serialize + for<'de> Deserialize<'de>,
{
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
                    let stringified_params = serde_json::to_string(params)
                        .unwrap_or_else(|_| "Failed to serialize params".to_string());
                    if let Err(e) = send_message(tx, stringified_params.into()).await {
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
