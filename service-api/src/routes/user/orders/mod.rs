use axum::{Router, routing::post};

use crate::state::AppState;

pub mod create_order;

pub fn router() -> Router<AppState> {
    Router::new().route("/create", post(create_order::create_order))
}
