use std::sync::Arc;

use crate::state::AppState;

pub async fn ws_handler(app_state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: handle websocket messages for maintaining order book updates on client side
    Ok(())
}
