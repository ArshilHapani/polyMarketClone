use axum::{Router, routing::get};

use crate::state::AppState;

pub mod get_markets;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_markets::get_markets))
        .route("/getAll", get(get_markets::get_markets))
}
