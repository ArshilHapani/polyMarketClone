use db_service::schema::market::Market as SchemaMarket;
use tonic::{Request, Response, Status};

use crate::{
    generated::{
        common::PageRequest,
        markets::{GetPaginatedMarketResponse, market_service_server::MarketService},
    },
    state::SafeState,
    validate_numbers,
};

pub struct MarketServiceStub {
    pub state: SafeState,
}

#[tonic::async_trait]
impl MarketService for MarketServiceStub {
    async fn get_market_data(
        &self,
        req: Request<PageRequest>,
    ) -> Result<Response<GetPaginatedMarketResponse>, Status> {
        let page_no = req.get_ref().page;
        let page_size = req.get_ref().page_size;
        validate_numbers!(page_no);
        validate_numbers!(page_size);

        let markets =
            SchemaMarket::get_all_markets_paginated(&self.state.db_pool, page_no, page_size)
                .await
                .map_err(|e| Status::internal(format!("Failed to get market {e}")))?;

        let response = GetPaginatedMarketResponse {
            markets: markets.items.iter().map(|item| item.into()).collect(),
            page_info: Some(markets.page_info.into()),
        };

        Ok(Response::new(response))
    }
}
