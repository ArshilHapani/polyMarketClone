use auth_service::types::SessionTokenClaims;
use axum::{Extension, extract::State, http::StatusCode, response::IntoResponse};

use crate::state::AppState;

pub async fn get_profile(
    State(_app_state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
) -> Result<impl IntoResponse, StatusCode> {
    println!("Claims: {:?}", claims);
    Ok("User profile information")
}
