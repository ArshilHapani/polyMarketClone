use db_service::schema::{
    enums::{OrderSide, OrderStatus, Outcome},
    orders::Order,
};
use rust_decimal::Decimal;
use uuid::Uuid;

use super::outcome_order_book::OutcomeOrderBook;

#[derive(Debug, Default)]
pub(crate) struct MarketOrderBook {
    yes_book: OutcomeOrderBook,
    no_book: OutcomeOrderBook,
    pub(crate) market_id: Uuid,
    pub(crate) current_yes_price: Decimal,
    pub(crate) current_no_price: Decimal,
    pub(crate) liquidity_b: Decimal,
}

impl MarketOrderBook {
    pub(crate) fn new(market_id: Uuid, liquidity_b: Decimal) -> Self {
        Self {
            yes_book: OutcomeOrderBook::default(),
            no_book: OutcomeOrderBook::default(),
            market_id,
            current_yes_price: Decimal::new(5, 1),
            current_no_price: Decimal::new(5, 1),
            liquidity_b,
        }
    }

    pub(crate) fn get_order_book(&mut self, outcome: &Outcome) -> Option<&mut OutcomeOrderBook> {
        match outcome {
            Outcome::YES => Some(&mut self.yes_book),
            Outcome::NO => Some(&mut self.no_book),
            _ => None,
        }
    }

    pub(crate) fn add_order(&mut self, order: &Order) {
        match order.outcome {
            Outcome::YES => self.yes_book.add_order(order),
            Outcome::NO => self.no_book.add_order(order),
            _ => {}
        };

        self.update_market_prices();
    }

    pub(crate) fn process_order(
        &mut self,
        order: &mut Order,
    ) -> Vec<(Uuid, Uuid, Decimal, Decimal)> {
        let matches = match order.outcome {
            Outcome::YES => self.yes_book.match_order(order),
            Outcome::NO => self.no_book.match_order(order),
            _ => Vec::new(),
        };

        if order.status == OrderStatus::OPEN {
            self.add_order(order);
        }
        self.update_market_prices();
        matches
    }

    pub(crate) fn update_market_prices(&mut self) {
        let funds_yes = self.calculate_total_funds(Outcome::YES);
        let funds_no = self.calculate_total_funds(Outcome::NO);

        if self.liquidity_b > Decimal::ZERO {
            self.current_yes_price = self.liquidity_b / (self.liquidity_b + funds_no);
            self.current_no_price = self.liquidity_b / (self.liquidity_b + funds_yes);
        } else {
            if let (Some(best_bid), Some(best_ask)) =
                (self.yes_book.best_bid(), self.yes_book.best_ask())
            {
                self.current_yes_price = (best_bid + best_ask) / Decimal::new(2, 0);
            }

            if let (Some(best_bid), Some(best_ask)) =
                (self.no_book.best_bid(), self.no_book.best_ask())
            {
                self.current_no_price = (best_bid + best_ask) / Decimal::new(2, 0);
            }
        }
    }

    pub(crate) fn calculate_total_funds(&self, outcome: Outcome) -> Decimal {
        match outcome {
            Outcome::YES => self
                .yes_book
                .bids
                .iter()
                .map(|(p, l)| *p * l.total_quantity)
                .sum(),
            Outcome::NO => self
                .no_book
                .bids
                .iter()
                .map(|(p, l)| *p * l.total_quantity)
                .sum(),
            _ => Decimal::ZERO,
        }
    }

    pub(crate) fn remove_order(
        &mut self,
        order_id: Uuid,
        side: OrderSide,
        outcome: Outcome,
        price: Decimal,
    ) -> bool {
        let result = match outcome {
            Outcome::YES => self.yes_book.remove_order(order_id, side, price),
            Outcome::NO => self.no_book.remove_order(order_id, side, price),
            _ => false,
        };
        if result {
            self.update_market_prices();
        }
        result
    }

    pub(crate) fn update_order(
        &mut self,
        order_id: Uuid,
        side: OrderSide,
        outcome: Outcome,
        price: Decimal,
        new_filled_quantity: Decimal,
    ) -> bool {
        let result = match outcome {
            Outcome::YES => self
                .yes_book
                .update_order(order_id, side, price, new_filled_quantity),
            Outcome::NO => self
                .no_book
                .update_order(order_id, side, price, new_filled_quantity),
            _ => false,
        };
        if result {
            self.update_market_prices();
        }
        result
    }
}
