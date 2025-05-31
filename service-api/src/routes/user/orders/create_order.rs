use std::str::FromStr;

use async_nats::jetstream;
use auth_service::types::SessionTokenClaims;
use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::{
    enums::{OrderSide, OrderStatus, Outcome},
    orders::Order,
};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::Uuid;
use utility_helpers::{log_error, log_info};

use crate::{require_field, state::AppState};

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateOrderPayload {
    market_id: Option<String>,
    price: Option<f64>,
    quantity: Option<f64>,
    side: Option<String>,
    outcome_side: Option<String>,
}

pub async fn create_order(
    State(app_state): State<AppState>,
    Extension(claims): Extension<SessionTokenClaims>,
    Json(payload): Json<CreateOrderPayload>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    require_field!(payload.market_id);
    require_field!(payload.price);
    require_field!(payload.quantity);
    require_field!(payload.side);
    require_field!(payload.outcome_side);

    let side: OrderSide = match payload.side.as_deref().unwrap().to_lowercase().as_str() {
        "buy" => OrderSide::BUY,
        "sell" => OrderSide::SELL,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid order side - must be 'buy' or 'sell'"
                }))
                .into_response(),
            ));
        }
    };
    let outcome_side: Outcome = match payload
        .outcome_side
        .as_deref()
        .unwrap()
        .to_lowercase()
        .as_str()
    {
        "yes" => Outcome::YES,
        "no" => Outcome::NO,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error":"Invalid outcome side - must be 'yes' or 'no'"
                }))
                .into_response(),
            ));
        }
    };

    let user_id = Uuid::from_str(&claims.user_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid user ID"
            }))
            .into_response(),
        )
    })?;
    let market_id = Uuid::from_str(&payload.market_id.unwrap()).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid market ID"
            }))
            .into_response(),
        )
    })?;
    let price = payload.price.unwrap();
    if price < 0f64 || price > 1f64 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Price must be between 0 and 1"
            }))
            .into_response(),
        ));
    }

    // asserting the channel exists
    app_state
        .jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: "ORDERS".into(),
            subjects: vec!["orders.>".into()],
            ..Default::default()
        })
        .await
        .map_err(|e| {
            log_error!("Failed to create jetstream stream - {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to create jetstream stream"
                }))
                .into_response(),
            )
        })?;

    let order = Order::create_order(
        user_id,
        market_id,
        from_f64(payload.price),
        from_f64(payload.quantity),
        side,
        outcome_side,
        &app_state.pg_pool,
    )
    .await
    .map_err(|e| {
        log_error!("Failed to create order - {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to create order"
            }))
            .into_response(),
        )
    })?;

    let mut is_failed = false;

    // pushing the order to the jetstream
    let order_id_str = order.id.to_string().into_bytes();

    app_state
        .jetstream
        .publish("orders.created".to_string(), order_id_str.into())
        .await
        .map_err(|e| {
            log_error!("Failed to publish order to jetstream - {:?}", e);
            // delete the order from the database if publishing fails
            is_failed = true;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to publish order to jetstream, order will be deleted"
                }))
                .into_response(),
            )
        })?;

    // if the order is failed, delete it from the database
    if is_failed {
        Order::update_order_status(order.id, OrderStatus::CANCELLED, &app_state.pg_pool)
            .await
            .map_err(|e| {
                log_error!("Failed to delete order - {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Failed to update order status to cancelled"
                    }))
                    .into_response(),
                )
            })?;
    }

    log_info!("Order published to jetstream - {:?}", order.id);

    let response = json!({
        "message": "Order created successfully",
        "order" : {
            "id": order.id,
            "user_id": order.user_id,
            "market_id": order.market_id,
            "side": order.side,
            "outcome": order.outcome,
            "price": order.price.to_string(),
            "quantity": order.quantity.to_string(),
            "filled_quantity": order.filled_quantity.to_string(),
            "status": order.status,
        }
    });

    Ok((StatusCode::CREATED, Json(response)))
}

fn from_f64(value: Option<f64>) -> Decimal {
    let value = value.unwrap();
    Decimal::from_f64(value)
        .unwrap_or_else(|| panic!("Failed to convert f64 to Decimal: {}", value))
}
