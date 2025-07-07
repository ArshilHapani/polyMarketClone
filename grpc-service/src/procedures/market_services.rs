use std::str::FromStr;

use db_service::schema::market::{self, Market as SchemaMarket};
use sqlx::types::Uuid;
use tonic::{Request, Response, Status};
use utility_helpers::redis::keys::RedisKey;

use crate::{
    generated::{
        common::PageRequest,
        markets::{
            GetMarketBookResponse, GetPaginatedMarketResponse, Market, OrderBook, OrderLevel,
            RequestForMarketBook, RequestWithMarketId, market_service_server::MarketService,
        },
    },
    procedures::from_db_market,
    state::SafeState,
    utils::clickhouse_schema::GetOrderBook,
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

        let key = RedisKey::Markets(page_no, page_size);
        if page_no == 0 || page_size == 0 {
            return Err(Status::invalid_argument(
                "Page number and size must be greater than 0",
            ));
        }

        let markets =
            self.state
                .redis_helper
                .get_or_set_cache(key, || async {
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
        req: Request<RequestWithMarketId>,
    ) -> Result<Response<Market>, Status> {
        let market_id = req.into_inner().market_id;
        validate_strings!(market_id);

        let market_id = Uuid::from_str(&market_id)
            .map_err(|_| Status::invalid_argument("Invalid market id"))?;

        let key = RedisKey::Market(market_id);

        let market = self
            .state
            .redis_helper
            .get_or_set_cache(key, || async {
                Ok(market::Market::get_market_by_id(&self.state.db_pool, &market_id).await?)
            })
            .await
            .map_err(|e| Status::internal(format!("Failed to get market id {e}")))?;

        // fetch volume from clickhouse

        if let Some(market) = market {
            let response = Response::new(from_db_market(&market, 0.5, 0.5));
            return Ok(response);
        }

        Err(Status::not_found(format!(
            "Market with {market_id} not found"
        )))
    }

    async fn get_market_book(
        &self,
        req: Request<RequestForMarketBook>,
    ) -> Result<Response<GetMarketBookResponse>, Status> {
        let market_id = &req.get_ref().market_id;
        let depth = req.get_ref().depth;
        validate_numbers!(depth);
        validate_strings!(market_id);

        let market_id = Uuid::from_str(&market_id)
            .map_err(|_| Status::invalid_argument("Invalid market id"))?;

        let order_book_initials = self
            .state
            .clickhouse_client
            .query(
                r#"
                SELECT
                    market_id,
                    ts,
                    created_at,

                    CAST(arraySlice(
                        arrayFilter(x -> x.2 > 0, yes_bids), 1, ?
                        ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS yes_bids,
                    CAST(arraySlice(
                        arrayFilter(x -> x.2 > 0, yes_asks), 1, ?
                        ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS yes_asks,
                    CAST(arraySlice(
                        arrayFilter(x -> x.2 > 0, no_bids), 1, ?
                        ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS no_bids,
                    CAST(arraySlice(
                        arrayFilter(x -> x.2 > 0, no_asks), 1, ?
                        ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS no_asks
                FROM market_order_book WHERE market_id = ?
                ORDER BY ts DESC
                LIMIT 1
            "#,
            )
            .bind(depth)
            .bind(depth)
            .bind(depth)
            .bind(depth)
            .bind(market_id)
            .fetch_optional::<GetOrderBook>()
            .await
            .map_err(|e| Status::internal(format!("Failed to fetch market book: {}", e)))?;

        if order_book_initials.is_none() {
            return Err(Status::not_found(format!(
                "Market book for market id {market_id} not found"
            )));
        }

        let order_book = to_resp(order_book_initials.unwrap());
        let response = Response::new(order_book);

        Ok(response)
    }
}

fn to_resp(data: GetOrderBook) -> GetMarketBookResponse {
    GetMarketBookResponse {
        market_id: data.market_id.to_string(),
        yes_book: Some(OrderBook {
            bids: data
                .yes_bids
                .into_iter()
                .map(|(price, shares, users)| OrderLevel {
                    price,
                    shares,
                    users,
                })
                .collect(),
            asks: data
                .yes_asks
                .into_iter()
                .map(|(price, shares, users)| OrderLevel {
                    price,
                    shares,
                    users,
                })
                .collect(),
        }),
        no_book: Some(OrderBook {
            bids: data
                .no_bids
                .into_iter()
                .map(|(price, shares, users)| OrderLevel {
                    price,
                    shares,
                    users,
                })
                .collect(),
            asks: data
                .no_asks
                .into_iter()
                .map(|(price, shares, users)| OrderLevel {
                    price,
                    shares,
                    users,
                })
                .collect(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use sqlx::types::Uuid;

    use crate::utils::clickhouse_schema::GetOrderBook;

    #[tokio::test]
    #[ignore = "Requires market id"]
    async fn test_get_market_data() {
        let client = clickhouse::Client::default()
            .with_url("http://localhost:8123")
            .with_database("polyMarket")
            .with_user("polyMarket")
            .with_password("polyMarket");
        let market_id = Uuid::from_str("91afed7f-6004-4968-984f-cdc968ae6013").unwrap();
        let depth = 10;

        let resp = client
            .query(
                r#"
                 SELECT
                    market_id,
                    ts,
                    created_at,

                    CAST(arraySlice(
                        arrayFilter(x -> x.2 > 0, yes_bids), 1, ?
                        ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS yes_bids,
                    CAST(arraySlice(
                        arrayFilter(x -> x.2 > 0, yes_asks), 1, ?
                        ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS yes_asks,
                    CAST(arraySlice(
                        arrayFilter(x -> x.2 > 0, no_bids), 1, ?
                        ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS no_bids,
                    CAST(arraySlice(
                        arrayFilter(x -> x.2 > 0, no_asks), 1, ?
                        ) AS Array(Tuple(price Float64, shares Float64, users UInt32))) AS no_asks
                FROM market_order_book WHERE market_id = ?
                ORDER BY ts DESC
                LIMIT 1
            "#,
            )
            .bind(depth)
            .bind(depth)
            .bind(depth)
            .bind(depth)
            .bind(market_id)
            .fetch_optional::<GetOrderBook>()
            .await
            .inspect_err(|e| {
                println!("Error fetching market data: {}", e);
            })
            .unwrap();

        assert!(resp.is_some(), "Response should not be empty");
    }
}
