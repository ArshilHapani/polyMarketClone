use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use db_service::schema::market::Market;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde_json::json;
use utility_helpers::log_error;

use crate::{require_fields_raw_response, state::AppState};

#[derive(serde::Deserialize)]
pub struct CreateMarketRequest {
    name: Option<String>,
    description: Option<String>,
    logo: Option<String>,
    liquidity_b: Option<f64>,
}

// Add market expiry in db
pub async fn create_new_market(
    State(state): State<AppState>,
    Json(payload): Json<CreateMarketRequest>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    require_fields_raw_response!(payload.name);
    require_fields_raw_response!(payload.description);
    require_fields_raw_response!(payload.logo);
    require_fields_raw_response!(payload.liquidity_b);

    let liquidity_b = payload.liquidity_b.unwrap();
    let name = payload.name.unwrap();
    let description = payload.description.unwrap();
    let logo = payload.logo.unwrap();

    let liquidity_b = Decimal::from_f64(liquidity_b).ok_or_else(|| {
        log_error!("Invalid liquidity_b value: {}", liquidity_b);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid liquidity_b value"
            })),
        )
    })?;

    let market = Market::create_new_market(name, description, logo, liquidity_b, &state.pg_pool)
        .await
        .map_err(|e| {
            log_error!("Error creating market: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to create market"
                })),
            )
        })?;

    let response = json!({
        "message": "Market created successfully",
        "market": {
            "id": market.id,
            "name": market.name,
            "description": market.description,
            "logo": market.logo,
            "liquidity_b": market.liquidity_b,
        }
    });
    Ok((StatusCode::CREATED, Json(response)).into_response())
}

pub fn router() -> Router<AppState> {
    Router::new().route("/create", post(create_new_market))
}
