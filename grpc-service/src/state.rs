use std::sync::Arc;

pub type SafeState = Arc<AppState>;
pub struct AppState {}

impl AppState {
    pub fn new() -> Self {
        AppState {}
    }
}
