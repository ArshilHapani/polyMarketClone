use db_service::{pagination::PageInfo as DbPageInfo, schema::market::Market as DbMarket};

use crate::{
    generated::{common::PageInfo, markets::Market},
    utils::to_f64,
};

pub mod market_service;

// all type conversations.....

impl From<&DbMarket> for Market {
    fn from(value: &DbMarket) -> Self {
        Market {
            created_at: value.created_at.to_string(),
            description: value.description.clone(),
            final_outcome: value.final_outcome as i32,
            id: value.id.to_string(),
            liquidity_b: to_f64(value.liquidity_b),
            logo: value.logo.clone(),
            name: value.name.clone(),
            status: value.status as i32,
            updated_at: value.updated_at.to_string(),
        }
    }
}

impl From<DbPageInfo> for PageInfo {
    fn from(value: DbPageInfo) -> Self {
        PageInfo {
            page: value.page,
            page_size: value.page_size,
            total_items: value.total_items,
            total_pages: value.total_pages,
        }
    }
}
