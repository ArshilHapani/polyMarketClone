use axum::{Router, routing::get};
use db_service::{SHOW_LOGS, log_info};
use routes::default_home_route;

mod routes;

const PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let addr = format!("[::]:{}", PORT);

    let app = Router::new().route("/", get(default_home_route));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    log_info!("service-api is listening on http://localhost:{}", PORT);

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
