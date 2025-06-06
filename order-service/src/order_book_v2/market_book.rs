use db_service::schema::{
    enums::{OrderSide, OrderStatus, Outcome},
    orders::Order,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::order_book_v2::outcome_book::OrderBookMatchedOutput;

use super::outcome_book::OutcomeBook;

#[derive(Debug)]
pub(crate) struct MarketBook {
    yes_order_book: OutcomeBook,
    no_order_book: OutcomeBook,

    pub(crate) current_yes_price: Decimal,
    pub(crate) current_no_price: Decimal,

    /// Liquidity parameter of the market
    ///
    /// The higher `b` = more liquidity, slower price changes
    pub(crate) liquidity_b: Decimal,
}

impl MarketBook {
    pub(super) fn new(liquidity_b: Decimal) -> Self {
        Self {
            yes_order_book: OutcomeBook::default(),
            no_order_book: OutcomeBook::default(),

            current_no_price: Decimal::new(5, 1),  // initial 0.5
            current_yes_price: Decimal::new(5, 1), // initial 0.5
            liquidity_b,
        }
    }

    pub(super) fn add_order(&mut self, order: &Order) {
        match order.outcome {
            Outcome::YES => self.yes_order_book.add_order(order),
            Outcome::NO => self.no_order_book.add_order(order),
            _ => {}
        }
        self.update_market_price();
    }

    pub(super) fn process_order(&mut self, order: &mut Order) -> Vec<OrderBookMatchedOutput> {
        let matches = match order.outcome {
            Outcome::YES => self.yes_order_book.match_order(order),
            Outcome::NO => self.no_order_book.match_order(order),
            _ => Vec::new(),
        };

        if order.status == OrderStatus::OPEN {
            self.add_order(order);
        }
        self.update_market_price();
        matches
    }

    pub(super) fn update_order(
        &mut self,
        order_id: Uuid,
        side: OrderSide,
        outcome: Outcome,
        price: Decimal,
        new_filled_quantity: Decimal,
    ) -> bool {
        let result = match outcome {
            Outcome::YES => {
                self.yes_order_book
                    .update_order(order_id, side, price, new_filled_quantity)
            }
            Outcome::NO => {
                self.no_order_book
                    .update_order(order_id, side, price, new_filled_quantity)
            }
            _ => false,
        };
        if result {
            self.update_market_price();
        }
        result
    }

    pub(super) fn remove_order(
        &mut self,
        order_id: Uuid,
        side: OrderSide,
        outcome: Outcome,
        price: Decimal,
    ) -> bool {
        let result = match outcome {
            Outcome::YES => self.yes_order_book.remove_order(order_id, side, price),
            Outcome::NO => self.no_order_book.remove_order(order_id, side, price),
            _ => false,
        };

        if result {
            self.update_market_price();
        }

        result
    }

    pub(crate) fn get_order_book(&mut self, outcome: Outcome) -> Option<&mut OutcomeBook> {
        match outcome {
            Outcome::YES => Some(&mut self.yes_order_book),
            Outcome::NO => Some(&mut self.no_order_book),
            _ => None,
        }
    }

    ///// Helpers //////

    fn update_market_price(&mut self) {
        // https://www.cultivatelabs.com/crowdsourced-forecasting-guide/how-does-logarithmic-market-scoring-rule-lmsr-work
        // Refer above blogpost for better understanding on LMSR (Logarithmic Market Scoring Rule) price mechanism for prediction markets
        if self.liquidity_b > Decimal::ZERO {
            let funds_yes = self.calculate_total_funds(Outcome::YES);
            let funds_no = self.calculate_total_funds(Outcome::NO);

            let total_liquidity = self.liquidity_b * dec!(2); // 2 * b for both sides
            let total_funds = funds_yes + funds_no;

            if total_funds > Decimal::ZERO {
                let yes_weight = (self.liquidity_b + funds_yes) / (total_liquidity + total_funds);
                let no_weight = (self.liquidity_b + funds_no) / (total_liquidity + total_funds);

                let total_weight = yes_weight + no_weight;
                self.current_yes_price = yes_weight / total_weight;
                self.current_no_price = no_weight / total_weight;
            } else {
                self.current_yes_price = dec!(0.5);
                self.current_no_price = dec!(0.5);
            }
        } else {
            let yes_mid = self.calculate_midpoint_price(&self.yes_order_book);
            let no_mid = self.calculate_midpoint_price(&self.no_order_book);

            match (yes_mid, no_mid) {
                (Some(yes_price), Some(no_price)) => {
                    let total = yes_price + no_price;
                    if total > Decimal::ZERO {
                        self.current_yes_price = yes_price / total;
                        self.current_no_price = no_price / total;
                    } else {
                        self.current_yes_price = dec!(0.5);
                        self.current_no_price = dec!(0.5);
                    }
                }

                (Some(yes_price), None) => {
                    self.current_yes_price = yes_price.min(dec!(0.95)); // cap at 0.95
                    self.current_no_price = dec!(1) - self.current_yes_price;
                }
                (None, Some(no_price)) => {
                    self.current_no_price = no_price.min(dec!(0.95)); // cap at 0.95
                    self.current_yes_price = dec!(1) - self.current_no_price;
                }
                (None, None) => {
                    self.current_yes_price = dec!(0.5);
                    self.current_no_price = dec!(0.5);
                }
            }
        }
    }

    fn calculate_total_funds(&self, outcome: Outcome) -> Decimal {
        // iterating over bids, because buyers have put their money. sellers are putting stocks (not money, so funds = bids for this part)
        match outcome {
            Outcome::YES => self
                .yes_order_book
                .bids
                .iter()
                .map(|(p, price_level)| *p * price_level.total_quantity)
                .sum(),
            Outcome::NO => self
                .no_order_book
                .bids
                .iter()
                .map(|(p, price_level)| *p * price_level.total_quantity)
                .sum(),
            _ => Decimal::ZERO,
        }
    }

    fn calculate_midpoint_price(&self, order_book: &OutcomeBook) -> Option<Decimal> {
        match (order_book.best_ask(), order_book.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / dec!(2)),
            (Some(bid), None) => Some(bid),
            (None, Some(ask)) => Some(ask),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDateTime;

    fn get_created_at() -> NaiveDateTime {
        chrono::Utc::now().naive_local()
    }
    fn get_random_uuid() -> Uuid {
        Uuid::new_v4()
    }

    #[test]
    fn test_create_market_book() {
        let liquidity_b = Decimal::new(100, 0); // 100 units of liquidity

        let market_book = MarketBook::new(liquidity_b);

        assert_eq!(market_book.liquidity_b, liquidity_b);
        assert_eq!(market_book.current_yes_price, Decimal::new(5, 1)); // 0.5
        assert_eq!(market_book.current_no_price, Decimal::new(5, 1)); // 0.5
        assert!(market_book.yes_order_book.bids.is_empty());
        assert!(market_book.no_order_book.bids.is_empty());
        assert!(market_book.yes_order_book.asks.is_empty());
        assert!(market_book.no_order_book.asks.is_empty());
    }

    #[test]
    fn test_add_order_and_price_update() {
        let order_1 = Order {
            id: get_random_uuid(),
            outcome: Outcome::YES,
            side: OrderSide::BUY,
            price: Decimal::new(5, 1),     // 0.5
            quantity: Decimal::new(10, 0), // 10 units
            status: OrderStatus::OPEN,
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            market_id: get_random_uuid(),
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };
        let order_2 = Order {
            id: get_random_uuid(),
            outcome: Outcome::NO,
            side: OrderSide::BUY,
            price: Decimal::new(5, 1),     // 0.5
            quantity: Decimal::new(10, 0), // 10 units
            status: OrderStatus::OPEN,
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            market_id: get_random_uuid(),
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let liquidity_b = Decimal::new(100, 0);
        let mut market_book = MarketBook::new(liquidity_b); // 100 units of liquidity

        market_book.add_order(&order_1);
        market_book.add_order(&order_1);
        market_book.add_order(&order_2);
        market_book.add_order(&order_2);

        assert_eq!(market_book.yes_order_book.bids.len(), 1);
        assert!(market_book.yes_order_book.bids.contains_key(&order_1.price));
        assert_eq!(
            market_book
                .yes_order_book
                .bids
                .get(&order_1.price)
                .unwrap()
                .total_quantity,
            order_1.quantity * dec!(2)
        );
        assert_eq!(market_book.current_yes_price, Decimal::new(5, 1)); //  0.5
        assert_eq!(market_book.current_no_price, Decimal::new(5, 1)); // 0.5

        market_book.add_order(&order_2); // adding another order on NO side to skew the price

        assert_ne!(market_book.current_yes_price, Decimal::new(5, 1)); // != 0.5
        assert_ne!(market_book.current_no_price, Decimal::new(5, 1)); // != 0.5
    }

    #[test]
    fn test_process_order() {
        let mut buy_order_1_yes = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id: get_random_uuid(),
            outcome: Outcome::YES,
            price: Decimal::new(50, 2), // 0.5
            quantity: Decimal::new(10, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut sell_order_1_yes = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id: get_random_uuid(),
            outcome: Outcome::YES,
            price: Decimal::new(50, 2), // 0.5
            quantity: Decimal::new(5, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut buy_order_1_no = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id: get_random_uuid(),
            outcome: Outcome::NO,
            price: Decimal::new(50, 2), // 0.5
            quantity: Decimal::new(10, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut sell_order_1_no = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id: get_random_uuid(),
            outcome: Outcome::NO,
            price: Decimal::new(50, 2), // 0.5
            quantity: Decimal::new(5, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut market_book = MarketBook::new(dec!(100));

        market_book.process_order(&mut buy_order_1_no);
        market_book.process_order(&mut buy_order_1_yes);

        let match_1 = market_book.process_order(&mut sell_order_1_no);
        let match_2 = market_book.process_order(&mut sell_order_1_yes);

        assert_eq!(match_1.len(), 1);
        assert_eq!(match_2.len(), 1);
        assert_eq!(match_1.get(0).unwrap().order_id, sell_order_1_no.id);
        assert_eq!(match_2.get(0).unwrap().order_id, sell_order_1_yes.id);
        assert_eq!(match_1.get(0).unwrap().opposite_order_id, buy_order_1_no.id);
        assert_eq!(
            match_2.get(0).unwrap().opposite_order_id,
            buy_order_1_yes.id
        );
    }

    #[test]
    fn test_update_order() {
        let id = get_random_uuid();
        let price = dec!(0.4);

        let order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id,
            price,
            market_id: get_random_uuid(),
            outcome: Outcome::YES,
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut market_book = MarketBook::new(dec!(100));
        market_book.add_order(&order);

        let price_level = market_book.yes_order_book.bids.get(&price).unwrap();

        assert_eq!(price_level.total_quantity, dec!(10));

        market_book.update_order(id, OrderSide::BUY, Outcome::YES, price, dec!(5));

        let price_level = market_book.yes_order_book.bids.get(&price).unwrap();

        assert_eq!(price_level.total_quantity, dec!(5));
    }

    #[test]
    fn test_remove_order() {
        let id = get_random_uuid();
        let price = dec!(0.5);
        let order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id,
            market_id: get_random_uuid(),
            outcome: Outcome::YES,
            price, // 0.5
            quantity: Decimal::new(10, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut market_book = MarketBook::new(dec!(100));

        market_book.add_order(&order);

        assert_eq!(market_book.yes_order_book.bids.len(), 1);

        market_book.remove_order(id, OrderSide::BUY, Outcome::YES, price);

        assert_eq!(market_book.yes_order_book.bids.len(), 0);
    }

    #[test]
    fn test_partial_fill() {
        let market_id = get_random_uuid();

        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25),
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.20),
            quantity: dec!(5),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let matches = outcome_book.match_order(&mut sell_order);

        // Verify partial fill
        assert_eq!(sell_order.status, OrderStatus::FILLED);
        assert_eq!(sell_order.filled_quantity, dec!(5));
        assert_eq!(matches.len(), 1);

        // Check the buy order was partially filled
        let price_level = outcome_book.bids.get(&buy_order.price).unwrap();
        assert_eq!(price_level.total_quantity, dec!(5));
    }

    #[test]
    fn test_match_multiple_orders_at_same_price() {
        let market_id = get_random_uuid();

        // Create multiple buy orders at the same price
        let buy_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25),
            quantity: dec!(5),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let buy_order_2 = Order {
            created_at: get_created_at()
                .checked_add_signed(chrono::Duration::seconds(1))
                .unwrap(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25), // Same price
            quantity: dec!(5),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.20),
            quantity: dec!(8),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order_1);
        outcome_book.add_order(&buy_order_2);

        let matches = outcome_book.match_order(&mut sell_order);

        // Verify time priority matching
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].opposite_order_id, buy_order_1.id); // First order matched first (time priority)
        assert_eq!(matches[1].opposite_order_id, buy_order_2.id);
        assert_eq!(sell_order.filled_quantity, dec!(8));
        assert_eq!(sell_order.status, OrderStatus::FILLED);

        // Check remaining quantity in order book
        let price_level = outcome_book.bids.get(&buy_order_1.price).unwrap();
        assert_eq!(price_level.total_quantity, dec!(2)); // 10 - 8 = 2 remaining
    }

    #[test]
    fn test_match_order_zero_quantity() {
        let market_id = get_random_uuid();

        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25),
            quantity: dec!(5),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.20),
            quantity: dec!(0), // Zero quantity
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let matches = outcome_book.match_order(&mut sell_order);

        // Verify no matches for zero quantity
        assert_eq!(matches.len(), 0);
        assert_eq!(sell_order.filled_quantity, dec!(0));
    }

    #[test]
    fn test_match_with_already_partially_filled_order() {
        let market_id = get_random_uuid();

        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25),
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(3), // Already partially filled
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.20),
            quantity: dec!(8),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let matches = outcome_book.match_order(&mut sell_order);

        // Verify correct matching considering previous fills
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_quantity, dec!(5)); // Only 5 more units matched (8-3)
        assert_eq!(sell_order.filled_quantity, dec!(8)); // 3 + 5 = 8
        assert_eq!(sell_order.status, OrderStatus::FILLED);
    }

    #[test]
    fn test_no_matching_price() {
        let market_id = get_random_uuid();

        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.20), // Lower than sell price
            quantity: dec!(5),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25), // Higher than buy price
            quantity: dec!(5),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let matches = outcome_book.match_order(&mut sell_order);

        // Verify no matches due to price mismatch
        assert_eq!(matches.len(), 0);
        assert_eq!(sell_order.filled_quantity, dec!(0));
        assert_eq!(sell_order.status, OrderStatus::OPEN);
    }

    #[test]
    fn test_remove_non_existent_order() {
        let mut outcome_book = OutcomeBook::default();

        // Try to remove an order that doesn't exist
        let result = outcome_book.remove_order(get_random_uuid(), OrderSide::BUY, dec!(0.5));
        assert!(!result);

        // Try to remove with wrong side
        let id = get_random_uuid();
        let order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id,
            price: dec!(0.5),
            market_id: get_random_uuid(),
            outcome: Outcome::YES,
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        outcome_book.add_order(&order);

        let result = outcome_book.remove_order(id, OrderSide::SELL, dec!(0.5));
        assert!(!result);
        assert_eq!(outcome_book.bids.len(), 1);
    }

    #[test]
    fn test_process_empty_book() {
        let market_id = get_random_uuid();
        let mut outcome_book = OutcomeBook::default();

        let mut order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.5),
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        // Process an order when book is empty
        let matches = outcome_book.match_order(&mut order);

        assert_eq!(matches.len(), 0);
        assert_eq!(order.filled_quantity, dec!(0));
        assert_eq!(order.status, OrderStatus::OPEN);
    }

    #[test]
    fn test_if_price_reaches_at_one_if_certain_range_of_order_hits() {
        let market_id = get_random_uuid();

        let buy_order_yes = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.5),
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let buy_order_no = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::NO,
            price: dec!(0.5),
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        };

        let mut market_book = MarketBook::new(dec!(100));
        for _ in 0..1000 {
            market_book.add_order(&buy_order_yes);
        }
        for _ in 0..100 {
            market_book.add_order(&buy_order_no);
        }

        println!(
            "yes price: {}\nNo price: {}",
            market_book.current_yes_price, market_book.current_no_price
        );
    }
}
