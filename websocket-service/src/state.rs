use crate::ws_utils::client_manager::ClientManager;

#[derive(Debug)]
pub struct WebSocketAppState {
    pub client_manager: ClientManager,
}

impl WebSocketAppState {
    pub fn new() -> Self {
        WebSocketAppState {
            client_manager: ClientManager::new(),
        }
    }
}
