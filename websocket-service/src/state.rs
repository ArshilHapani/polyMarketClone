use crate::ws_utils::{
    SubscriptionChannel, process_manager::ProcessManager, process_manager_v2::ProcessManagerV2,
};

#[derive(Debug)]
pub struct WebSocketAppState {
    pub process_manager_v2: ProcessManagerV2,
    pub process_manager: ProcessManager,
}

impl WebSocketAppState {
    pub fn new() -> Self {
        WebSocketAppState {
            process_manager_v2: ProcessManagerV2::new(),
            process_manager: ProcessManager::new(),
        }
    }

    pub fn cleanup_processes(&mut self, channel: &SubscriptionChannel) {
        self.process_manager_v2.delete_process(channel);
    }
}
