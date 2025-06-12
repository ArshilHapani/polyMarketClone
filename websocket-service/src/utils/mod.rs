use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, stream::SplitSink};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub mod client_manager;
pub mod handle_connection;
pub mod handle_message;
pub mod process_channel_request;

pub type SafeSender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

pub(super) async fn send_message(tx: &SafeSender, message: Message) -> Result<(), axum::Error> {
    tx.lock().await.send(message).await
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ChannelType {
    PriceUpdate,
    PricePoster,
}

impl ChannelType {
    pub fn to_string(&self) -> String {
        match self {
            ChannelType::PriceUpdate => "price_update".to_string(),
            ChannelType::PricePoster => "price_poster".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "price_update" => Some(ChannelType::PriceUpdate),
            "price_poster" => Some(ChannelType::PricePoster),
            _ => None,
        }
    }
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
    Post {
        channel: String,
        data: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    pub id: Option<String>,
    pub payload: MessagePayload,
}
