use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

pub async fn default_home_route() -> (StatusCode, impl IntoResponse) {
    let welcome_message = json!({
        "message": "Welcome to the Polymarket clone service API!"
    });
    (StatusCode::OK, Json(welcome_message))
}
