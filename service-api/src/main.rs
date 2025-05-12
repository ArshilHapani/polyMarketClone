use axum::Router;
use db_service::{SHOW_LOGS, log_info};

use state::AppState;

mod routes;
mod state;
mod utils;

const PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let addr = format!("[::]:{}", PORT);

    let state = AppState::new().await?;

    let app = Router::new()
        .merge(routes::router(state.clone()))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    log_info!("service-api is listening on http://localhost:{}", PORT);

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
