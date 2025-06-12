use tokio::sync::RwLock;

use crate::utils::client_manager::SubscriptionAndClientManager;

#[derive(Debug)]
pub struct WebSocketAppState {
    pub client_manager: RwLock<SubscriptionAndClientManager>,
}

impl WebSocketAppState {
    pub fn new() -> Self {
        WebSocketAppState {
            client_manager: RwLock::new(SubscriptionAndClientManager::new()),
        }
    }
}
