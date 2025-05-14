use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use axum_extra::protobuf::Protobuf;
use db_service::schema::{
    enums::{MarketStatus as MarketStatusDb, Outcome},
    market::Market as MarketSchema,
};
use service_api::generated::markets::{GetPaginatedMarketResponse, Market};

use crate::{state::AppState, utils::types::PaginationRequestQuery};

pub async fn get_markets(
    State(app_state): State<AppState>,
    Query(page_info): Query<PaginationRequestQuery>,
) -> Result<Protobuf<GetPaginatedMarketResponse>, StatusCode> {
    let markets_data_with_page_info = MarketSchema::get_all_markets_paginated(
        &app_state.pg_pool,
        page_info.page,
        page_info.page_size,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let markets: Vec<Market> = markets_data_with_page_info
        .items
        .into_iter()
        .map(|market| Market {
            id: market.id.to_string(),
            name: market.name,
            description: market.description,
            logo: market.logo,
            status: get_status_from_market_status(market.status),
            final_outcome: get_outcome_from_market_outcome(market.final_outcome),
            liquidity_b: market.liquidity_b.to_string().parse().unwrap_or_default(),
            created_at: market.created_at.to_string(),
            updated_at: market.updated_at.to_string(),
        })
        .collect();

    let response = GetPaginatedMarketResponse {
        markets,
        page: markets_data_with_page_info.page_info.page,
        page_size: markets_data_with_page_info.page_info.page_size,
        total_items: markets_data_with_page_info.page_info.total_items,
        total_pages: markets_data_with_page_info.page_info.total_pages,
    };

    Ok(Protobuf(response))
}

fn get_status_from_market_status(market_status: MarketStatusDb) -> i32 {
    match market_status {
        MarketStatusDb::OPEN => 1,
        MarketStatusDb::CLOSED => 2,
        MarketStatusDb::SETTLED => 3,
    }
}

fn get_outcome_from_market_outcome(market_outcome: Outcome) -> i32 {
    match market_outcome {
        Outcome::YES => 1,
        Outcome::NO => 2,
        Outcome::UNSPECIFIED => 3,
    }
}
