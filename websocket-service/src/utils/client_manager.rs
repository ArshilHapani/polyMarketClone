use std::collections::HashMap;

use uuid::Uuid;

use crate::utils::{ChannelType, SafeSender};

type ClientData = (SafeSender, serde_json::Value);

#[derive(Debug)]
pub struct SubscriptionAndClientManager {
    pub subscription: HashMap<ChannelType, HashMap<Uuid, ClientData>>,
}

impl SubscriptionAndClientManager {
    pub fn new() -> Self {
        Self {
            subscription: HashMap::new(),
        }
    }

    pub fn add_client(
        &mut self,
        channel: ChannelType,
        client_id: Uuid,
        tx: SafeSender,
        params: serde_json::Value,
    ) {
        self.subscription
            .entry(channel)
            .or_insert_with(HashMap::new)
            .insert(client_id, (tx, params));
    }

    pub fn remove_client(&mut self, channel: &ChannelType, client_id: &Uuid) {
        if let Some(clients) = self.subscription.get_mut(channel) {
            clients.remove(client_id);
            if clients.is_empty() {
                self.subscription.remove(&channel);
            }
        }
    }
    pub fn get_clients(&self, channel: &ChannelType) -> Option<&HashMap<Uuid, ClientData>> {
        self.subscription.get(channel)
    }

    pub fn cleanup(&mut self) {
        self.subscription.clear();
    }
}
