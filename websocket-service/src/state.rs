use crate::ws_utils::{SubscriptionChannel, process_manager_v2::ProcessManagerV2};

#[derive(Debug)]
pub struct WebSocketAppState {
    pub process_manager: ProcessManagerV2,
}

impl WebSocketAppState {
    pub fn new() -> Self {
        WebSocketAppState {
            process_manager: ProcessManagerV2::new(),
        }
    }

    pub fn cleanup_processes(&mut self, channel: &SubscriptionChannel) {
        self.process_manager.delete_process(channel);
    }
}
