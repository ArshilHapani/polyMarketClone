use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get};
use serde_json::json;

use crate::state::AppState;

pub mod admin;
pub mod user;

async fn default_home_route() -> (StatusCode, impl IntoResponse) {
    let welcome_message = json!({
        "message": "Welcome to the Polymarket clone service API!"
    });
    (StatusCode::OK, Json(welcome_message))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(default_home_route))
        .nest("/user", user::router())
}
