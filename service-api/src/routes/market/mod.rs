use axum::{Router, routing::get};

use crate::state::AppState;

pub async fn get_markets() -> &'static str {
    "Markets"
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_markets))
        .route("/getAll", get(get_markets))
}
