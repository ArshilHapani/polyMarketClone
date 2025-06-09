use std::{collections::HashMap, sync::Arc};

use axum::extract::ws::{Message, Utf8Bytes};
use futures::SinkExt;
use tokio::sync::RwLock;
use utility_helpers::{log_error, log_warn};
use uuid::Uuid;

use super::SubscriptionChannels;
use crate::ws_utils::SafeSender;

#[derive(Debug)]
pub struct Client {
    pub id: Uuid,
    pub sender: SafeSender,
}

#[derive(Debug, Clone)]
pub struct ClientManager {
    inner: Arc<RwLock<HashMap<SubscriptionChannels, HashMap<Uuid, Client>>>>,
}

impl ClientManager {
    pub fn new() -> Self {
        ClientManager {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub async fn add_client(
        &mut self,
        id: Uuid,
        channel: SubscriptionChannels,
        sender: SafeSender,
    ) {
        let client = Client {
            id,
            sender: sender.clone(),
        };
        let mut inner = self.inner.write().await;
        inner
            .entry(channel.clone())
            .or_insert_with(HashMap::new)
            .insert(id, client);

        let _ = sender
            .lock()
            .await
            .send(Message::Text(Utf8Bytes::from(format!(
                "Subscribed to channel {channel}"
            ))))
            .await;
    }

    pub async fn remove_client(&mut self, id: Uuid, channel: SubscriptionChannels) {
        let mut inner = self.inner.write().await;
        if let Some(clients) = inner.get_mut(&channel) {
            clients.remove(&id);
            if clients.is_empty() {
                inner.remove(&channel);
            }
        }
    }

    pub async fn broadcast(&self, channel: &SubscriptionChannels, message: String) {
        let inner = self.inner.read().await;
        if let Some(clients) = inner.get(channel) {
            for client in clients.values() {
                let sender = client.sender.clone();
                let msg = Message::Text(Utf8Bytes::from(message.clone()));
                let mut sender_lock = sender.lock().await;
                if let Err(e) = sender_lock.send(msg).await {
                    log_error!("Failed to send message to client {}: {}", client.id, e);
                }
            }
        } else {
            log_warn!("No clients found for channel {:?}", channel);
        }
    }
}
