use std::{fmt::Display, sync::Arc};

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use futures::stream::SplitSink;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

pub(crate) mod connection_handler;
pub(crate) mod handle_messages;
pub(crate) mod process_manager;
pub(crate) mod process_manager_v2;
pub(crate) mod tasks;

pub type SafeSender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum SubscriptionChannel {
    PriceUpdates(Uuid),    // market id
    OrderBookUpdate(Uuid), // market id
}

impl SubscriptionChannel {
    pub fn from_str(channel: &str) -> Option<Self> {
        if channel.starts_with("price_updates:") {
            let id_str = channel.trim_start_matches("price_updates:");
            if let Ok(id) = Uuid::parse_str(id_str) {
                return Some(SubscriptionChannel::PriceUpdates(id));
            }
        } else if channel.starts_with("order_book_update:") {
            let id_str = channel.trim_start_matches("order_book_update:");
            if let Ok(id) = Uuid::parse_str(id_str) {
                return Some(SubscriptionChannel::OrderBookUpdate(id));
            }
        }
        None
    }

    pub fn to_string(&self) -> String {
        match self {
            SubscriptionChannel::PriceUpdates(id) => format!("price_updates:{}", id),
            SubscriptionChannel::OrderBookUpdate(id) => format!("order_book_update:{}", id),
        }
    }
}

impl Display for SubscriptionChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscriptionChannel::PriceUpdates(id) => {
                let _ = f.write_str(format!("PriceUpdates({id})").as_str());
            }
            SubscriptionChannel::OrderBookUpdate(id) => {
                let _ = f.write_str(format!("OrderBookUpdate({id})").as_str());
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
