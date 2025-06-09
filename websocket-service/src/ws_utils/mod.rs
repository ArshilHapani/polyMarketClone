use std::{fmt::Display, sync::Arc};

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures::stream::SplitSink;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

pub(crate) mod client_manager;
pub(crate) mod connection_handler;
pub(crate) mod process_manager;

pub type SafeSender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum SubscriptionChannels {
    PriceUpdates(Uuid),
}

impl SubscriptionChannels {
    pub fn from_str(channel: &str) -> Option<Self> {
        if channel.starts_with("price_updates:") {
            let id_str = channel.trim_start_matches("price_updates:");
            if let Ok(id) = Uuid::parse_str(id_str) {
                return Some(SubscriptionChannels::PriceUpdates(id));
            }
        }
        None
    }

    pub fn to_string(&self) -> String {
        match self {
            SubscriptionChannels::PriceUpdates(id) => format!("price_updates:{}", id),
        }
    }
}

impl Display for SubscriptionChannels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscriptionChannels::PriceUpdates(id) => {
                let _ = f.write_str(format!("PriceUpdates({id})").as_str());
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum DisconnectReason {
    ClientClose(Option<CloseFrame>),
    ProtocolError(String),
    SendError(String),
    StreamEnded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessagePayload {
    Subscribe {
        channel: String,
        params: serde_json::Value,
    },
    Unsubscribe {
        channel: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    pub id: Option<String>,
    pub payload: MessagePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub channel: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
