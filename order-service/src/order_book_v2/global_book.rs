use std::collections::HashMap;

use uuid::Uuid;

use super::market_book::MarketBook;

#[derive(Debug)]
pub(crate) struct GlobalMarketBook {
    pub(crate) markets: HashMap<Uuid, MarketBook>,
}
