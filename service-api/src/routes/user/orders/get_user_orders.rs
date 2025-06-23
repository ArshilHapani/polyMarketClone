use std::str::FromStr;

use auth_service::types::SessionTokenClaims;
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use db_service::schema::orders::Order;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utility_helpers::log_error;
use uuid::Uuid;

use crate::{require_field, state::AppState, validate_paginated_fields};

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryParams {
    page: Option<u32>,
    page_size: Option<u32>,
}

pub async fn get_user_orders(
    State(app_state): State<AppState>,
    Query(params): Query<QueryParams>,
    Extension(claims): Extension<SessionTokenClaims>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;
    let user_id = Uuid::from_str(&user_id).unwrap(); // already validated in auth middleware

    require_field!(params.page);
    require_field!(params.page_size);

    let page = params.page.unwrap();
    let page_size = params.page_size.unwrap();

    validate_paginated_fields!(page, page_size);

    let user_orders =
        Order::get_user_orders_by_paginated(&app_state.pg_pool, user_id, page, page_size)
            .await
            .map_err(|e| {
                log_error!("Failed to fetch user orders {e:?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"message": "Failed to fetch user orders"})).into_response(),
                )
            })?;

    Ok(Json(json!({
        "orders": user_orders,
        "page": page,
        "page_size": page_size,
    }))
    .into_response())
}

pub async fn get_user_orders_by_market(
    State(app_state): State<AppState>,
    Query(params): Query<QueryParams>,
    Extension(claims): Extension<SessionTokenClaims>,
    Path(market_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Response)> {
    let user_id = claims.user_id;
    let user_id = Uuid::from_str(&user_id).unwrap(); // already validated in auth middleware

    require_field!(params.page);
    require_field!(params.page_size);

    let page = params.page.unwrap();
    let page_size = params.page_size.unwrap();

    validate_paginated_fields!(page, page_size);

    let user_orders = Order::get_user_orders_by_market_paginated(
        &app_state.pg_pool,
        user_id,
        market_id,
        page,
        page_size,
    )
    .await
    .map_err(|e| {
        log_error!("Failed to fetch user orders {e:?}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": "Failed to fetch user orders"})).into_response(),
        )
    })?;

    Ok(Json(json!({
        "orders": user_orders,
        "page": page,
        "page_size": page_size,
    }))
    .into_response())
}
