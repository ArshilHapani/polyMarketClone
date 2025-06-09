use axum::{
    Router,
    extract::{State, ws::WebSocketUpgrade},
    routing::any,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber;
use utility_helpers::log_info;

use crate::{state::WebSocketAppState, ws_utils::connection_handler::handle_connection};

mod state;
mod ws_utils;

pub type AppState = Arc<RwLock<WebSocketAppState>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app_state = Arc::new(RwLock::new(WebSocketAppState::new()));

    let app = Router::new()
        .route("/ws", any(socket_handler))
        .with_state(app_state.clone());

    let listener = tokio::net::TcpListener::bind("[::]:4010")
        .await
        .expect("Failed to bind TCP listener");

    log_info!("Starting WebSocket server on port 4040");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn socket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_connection(socket, state))
}
