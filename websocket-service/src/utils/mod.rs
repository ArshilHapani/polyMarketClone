use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, stream::SplitSink};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub mod client_manager;
pub mod handle_connection;
pub mod handle_message;
pub mod process_channel_request;

// mutex because rx.next() method requires mutable access, so one reader and writer at a time...
pub type SafeSender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

pub(super) async fn send_message(tx: &SafeSender, message: Message) -> Result<(), axum::Error> {
    tx.lock().await.send(message).await
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
    pub id: Option<String>, //TODO we can verify the client id with this id (TODO for now)
    pub payload: MessagePayload,
}
