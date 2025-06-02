use rust_decimal::Decimal;

use super::outcome_book::OutcomeBook;

#[derive(Debug)]
pub(crate) struct MarketBook {
    yes_order_book: OutcomeBook,
    no_order_book: OutcomeBook,

    pub(crate) current_yes_price: Decimal,
    pub(crate) current_no_price: Decimal,
    pub(crate) liquidity_b: Decimal,
}
