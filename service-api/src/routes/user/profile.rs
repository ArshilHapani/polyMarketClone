use auth_service::types::SessionTokenClaims;
use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::users::User;
use serde_json::json;
use utility_helpers::log_error;

use crate::state::AppState;

pub async fn get_profile(
    State(app_state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user = User::get_user_by_id(&app_state.pg_pool, claims.user_id)
        .await
        .map_err(|e| {
            log_error!("Failed to get user by ID: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to get user profile"
                }))
                .into_response(),
            )
        })?;

    let response = json!({
        "id": user.id,
        "google_id": user.google_id,
        "email": user.email,
        "name": user.name,
        "avatar": user.avatar,
        "public_key": user.public_key,
        "balance": user.balance,
    });

    Ok((StatusCode::OK, Json(response)))
}
