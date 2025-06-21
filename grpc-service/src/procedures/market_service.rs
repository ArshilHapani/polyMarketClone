use std::str::FromStr;

use db_service::schema::market::{self, Market as SchemaMarket};
use sqlx::types::Uuid;
use tonic::{Request, Response, Status};

use crate::{
    generated::{
        common::PageRequest,
        markets::{
            GetMarketByIdRequest, GetPaginatedMarketResponse, Market,
            market_service_server::MarketService,
        },
    },
    procedures::from_db_market,
    state::SafeState,
    validate_numbers, validate_strings,
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

        let key = format!("markets:{}:{}", page_no, page_size);
        if page_no == 0 || page_size == 0 {
            return Err(Status::invalid_argument(
                "Page number and size must be greater than 0",
            ));
        }

        let markets =
            self.state
                .redis_helper
                .get_or_set_cache(&key, || async {
                    Ok(SchemaMarket::get_all_markets_paginated(
                        &self.state.db_pool,
                        page_no,
                        page_size,
                    )
                    .await?)
                })
                .await
                .map_err(|e| Status::internal(format!("Failed to get market {e}")))?;

        let response = GetPaginatedMarketResponse {
            markets: markets
                .items
                .iter()
                .map(|item| from_db_market(item, 0.5, 0.5))
                .collect(),
            page_info: Some(markets.page_info.into()),
        };

        Ok(Response::new(response))
    }

    async fn get_market_by_id(
        &self,
        req: Request<GetMarketByIdRequest>,
    ) -> Result<Response<Market>, Status> {
        let market_id = req.into_inner().market_id;
        validate_strings!(market_id);

        let market_id = Uuid::from_str(&market_id)
            .map_err(|_| Status::invalid_argument("Invalid market id"))?;

        let market = self
            .state
            .redis_helper
            .get_or_set_cache("market", || async {
                Ok(market::Market::get_market_by_id(&self.state.db_pool, &market_id).await?)
            })
            .await
            .map_err(|e| Status::internal(format!("Failed to get market id {e}")))?;

        if let Some(market) = market {
            let response = Response::new(from_db_market(&market, 0.5, 0.5));

            return Ok(response);
        }

        Err(Status::not_found(format!(
            "Market with {market_id} not found"
        )))
    }
}
