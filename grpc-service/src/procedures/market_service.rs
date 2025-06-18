use tonic::{Request, Response, Status};

use crate::{
    markets_tonic::{Empty, MarketDataResponse, market_service_server::MarketService},
    state::SafeState,
};

pub struct MarketServiceStub {
    pub state: SafeState,
}

#[tonic::async_trait]
impl MarketService for MarketServiceStub {
    async fn get_market_data(
        &self,
        req: Request<Empty>,
    ) -> Result<Response<MarketDataResponse>, Status> {
        Err(Status::unimplemented("Not implemented yet"))
    }
}
