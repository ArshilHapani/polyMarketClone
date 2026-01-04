use crate::state::AppState;
use auth_service::types::SessionTokenClaims;
use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize, Debug)]
pub struct DepositRequest {
    pub amount: f64,
}

pub async fn deposit_funds_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Json(DepositRequest { amount }): Json<DepositRequest>,
) -> Result<Response, (StatusCode, Response)> {
    let user_id = claims.user_id;

    let amount_decimal = Decimal::from_f64(amount).ok_or_else(|| {
        Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid amount provided"
            }))
            .into_response(),
        ))
    })?;

    // just deposit balance for now

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Deposit successful",
            "user_id": user_id,
            "amount": amount_decimal.to_string()
        })),
    )
        .into_response())
}
