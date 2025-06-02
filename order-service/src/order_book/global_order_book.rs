use std::collections::HashMap;

use db_service::schema::{enums::Outcome, orders::Order};
use rust_decimal::Decimal;
use uuid::Uuid;

use super::market_order_book::MarketOrderBook;

#[derive(Debug)]
pub(crate) struct GlobalOrderBook {
    pub(crate) markets: HashMap<Uuid, MarketOrderBook>,
}

impl GlobalOrderBook {
    pub(crate) fn new() -> Self {
        Self {
            markets: HashMap::new(),
        }
    }

    pub(crate) fn get_or_create_market(
        &mut self,
        market_id: Uuid,
        liquidity_b: Decimal,
    ) -> &mut MarketOrderBook {
        self.markets
            .entry(market_id)
            .or_insert_with(|| MarketOrderBook::new(market_id, liquidity_b))
    }

    pub(crate) fn process_order(
        &mut self,
        order: &mut Order,
        liquidity_b: Decimal,
    ) -> Vec<(Uuid, Uuid, Decimal, Decimal)> {
        let market_id = order.market_id;
        let market_book = self.get_or_create_market(market_id, liquidity_b);
        market_book.process_order(order)
    }

    pub(crate) fn get_market_price(&self, market_id: &Uuid, outcome: Outcome) -> Option<Decimal> {
        self.markets.get(&market_id).map(|market| match outcome {
            Outcome::YES => market.current_yes_price,
            Outcome::NO => market.current_no_price,
            _ => Decimal::ZERO,
        })
    }
}
