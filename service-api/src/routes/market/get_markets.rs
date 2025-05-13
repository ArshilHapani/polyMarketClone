use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use axum_extra::protobuf::Protobuf;
use db_service::{pagination::PageInfo, schema::market::Market as MarketSchema};
use service_api::markets::{GetPaginatedMarketResponse, Market};

use crate::state::AppState;

pub async fn get_markets(
    State(app_state): State<AppState>,
    Query(page_info): Query<PageInfo>,
) -> Result<Protobuf<GetPaginatedMarketResponse>, StatusCode> {
    let markets = MarketSchema::get_all_markets_paginated(
        &app_state.pg_pool,
        page_info.page,
        page_info.page_size,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Err(StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS) // TODO
}
