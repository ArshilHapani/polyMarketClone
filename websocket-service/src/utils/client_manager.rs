use std::collections::HashMap;

use utility_helpers::ws::types::ChannelType;
use uuid::Uuid;

use crate::utils::SafeSender;

type ClientData = SafeSender;

#[derive(Debug, Hash, PartialEq)]
pub enum SpecialKindOfClients {
    OrderService,
}

#[derive(Debug)]
pub struct SubscriptionAndClientManager {
    subscription: HashMap<ChannelType, HashMap<Uuid, ClientData>>,
    special_clients: HashMap<Uuid, SpecialKindOfClients>,
}

impl SubscriptionAndClientManager {
    pub fn new() -> Self {
        Self {
            subscription: HashMap::new(),
            special_clients: HashMap::new(),
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

    pub fn set_special_client(&mut self, client_id: Uuid, kind: SpecialKindOfClients) -> bool {
        if !self.is_client_id_exist(&client_id) {
            return false;
        }
        self.special_clients.insert(client_id, kind);
        true
    }

    pub fn unset_special_client(&mut self, client_id: &Uuid) {
        self.special_clients.remove(client_id);
    }

    fn is_client_id_exist(&self, client_id: &Uuid) -> bool {
        for (_, clients) in self.subscription.iter() {
            for (uuid, _) in clients {
                if *client_id == *uuid {
                    return true;
                }
            }
        }
        false
    }
}
