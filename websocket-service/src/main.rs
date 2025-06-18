use axum::{
    Router,
    extract::{State, ws::WebSocketUpgrade},
    routing::any,
};
use std::sync::Arc;
use tracing_subscriber;
use utility_helpers::log_info;

use crate::{state::WebSocketAppState, utils::handle_connection::handle_connection};

mod state;
mod utils;

pub type SafeAppState = Arc<WebSocketAppState>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app_state = Arc::new(WebSocketAppState::new());

    let app = Router::new()
        .route("/", any(|| async { "Hello from WebSocket server!" }))
        .route("/ws", any(socket_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("[::]:4010")
        .await
        .expect("Failed to bind TCP listener");

    log_info!("Starting WebSocket server on port 4010");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn socket_handler(
    ws: WebSocketUpgrade,
    State(state): State<SafeAppState>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_connection(socket, state))
}
