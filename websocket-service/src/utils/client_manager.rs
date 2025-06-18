use std::collections::HashMap;

use utility_helpers::ws::types::ChannelType;
use uuid::Uuid;

use crate::utils::SafeSender;

type ClientData = SafeSender;

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

    pub fn add_client(&mut self, channel: ChannelType, client_id: Uuid, tx: SafeSender) {
        self.subscription
            .entry(channel)
            .or_insert_with(HashMap::new)
            .insert(client_id, tx);
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

    pub fn remove_client_without_channel(&mut self, client_id: &Uuid) {
        for (_, clients) in self.subscription.iter_mut() {
            if let Some(_) = clients.get(client_id) {
                clients.remove(client_id);
            }
        }
    }
}
