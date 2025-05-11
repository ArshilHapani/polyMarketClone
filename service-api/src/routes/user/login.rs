use axum::{extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {}

pub async fn login(State(app_state): State<AppState>) -> (StatusCode, impl IntoResponse) {
    (StatusCode::OK, "Login successful")
}
