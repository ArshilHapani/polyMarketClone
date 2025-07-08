use auth_service::types::SessionTokenClaims;
use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::state::AppState;

pub async fn create_limit_order(
    State(app_state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    Ok(Json(json!({
        "message": "Limit order creation is not implemented yet.",
        "user_id": claims.user_id,
    })))
}
