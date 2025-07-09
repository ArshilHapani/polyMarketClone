use db_service::schema::{
    enums::{OrderSide, OrderStatus, Outcome},
    orders::Order,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use crate::order_book::outcome_book::OrderBookMatchedOutput;

use super::outcome_book::OutcomeBook;

#[derive(Debug)]
pub(crate) struct MarketBook {
    yes_order_book: OutcomeBook,
    no_order_book: OutcomeBook,

    pub(crate) executed_yes_buy_volume: Decimal,
    pub(crate) executed_no_buy_volume: Decimal,

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

            executed_yes_buy_volume: Decimal::ZERO,
            executed_no_buy_volume: Decimal::ZERO,

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

        if order.status == OrderStatus::OPEN || order.status == OrderStatus::PendingUpdate {
            self.add_order(order);
        }
        self.update_market_price();
        matches
    }

    pub(super) fn create_market_order(
        &mut self,
        order: &mut Order,
        budget: Decimal,
    ) -> Vec<OrderBookMatchedOutput> {
        let matches = match order.outcome {
            Outcome::YES => self.yes_order_book.create_market_order(order, budget),
            Outcome::NO => self.no_order_book.create_market_order(order, budget),
            _ => Vec::new(),
        };

        if order.side == OrderSide::BUY && order.filled_quantity > Decimal::ZERO {
            let executed_value = matches
                .iter()
                .map(|m| m.price * m.matched_quantity)
                .sum::<Decimal>();

            match order.outcome {
                Outcome::YES => self.executed_yes_buy_volume += executed_value,
                Outcome::NO => self.executed_no_buy_volume += executed_value,
                _ => {}
            }
        }

        self.update_market_price();
        matches
    }

    pub(super) fn update_order(
        &mut self,
        order: &mut Order,
        new_quantity: Decimal,
        new_price: Decimal,
    ) -> bool {
        let result = match order.outcome {
            Outcome::YES => self
                .yes_order_book
                .update_order(order, new_price, new_quantity),
            Outcome::NO => self
                .no_order_book
                .update_order(order, new_price, new_quantity),
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

    pub(crate) fn get_order_book(&self, outcome: Outcome) -> Option<&OutcomeBook> {
        match outcome {
            Outcome::YES => Some(&self.yes_order_book),
            Outcome::NO => Some(&self.no_order_book),
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
        let book_funds = match outcome {
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
        };

        let executed_funds = match outcome {
            Outcome::YES => self.executed_yes_buy_volume,
            Outcome::NO => self.executed_no_buy_volume,
            _ => Decimal::ZERO,
        };

        book_funds + executed_funds
    }

    fn calculate_midpoint_price(&self, order_book: &OutcomeBook) -> Option<Decimal> {
        match (order_book.best_bid(), order_book.best_ask()) {
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
    use db_service::schema::enums::OrderType;

    fn get_created_at() -> NaiveDateTime {
        chrono::Utc::now().naive_local()
    }
    fn get_random_uuid() -> Uuid {
        Uuid::new_v4()
    }
    #[test]
    fn test_market_order_empty_book_behavior() {
        let mut market_book = MarketBook::new(dec!(100));
        let market_id = Uuid::new_v4();

        let mut market_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),
            quantity: dec!(0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::MARKET,
        };

        let budget = dec!(100); // Large budget but empty book
        let matches = market_book.create_market_order(&mut market_order, budget);

        // Results of empty book matching:
        assert_eq!(matches.len(), 0); // No matches
        assert_eq!(market_order.quantity, dec!(0)); // No quantity
        assert_eq!(market_order.filled_quantity, dec!(0)); // Nothing filled
        assert_eq!(market_order.status, OrderStatus::OPEN); // Still "filled"

        // Prices remain at default
        assert_eq!(market_book.current_yes_price, dec!(0.5));
        assert_eq!(market_book.current_no_price, dec!(0.5));

        // No executed volume tracked
        assert_eq!(market_book.executed_yes_buy_volume, dec!(0));
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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
            order_type: OrderType::LIMIT,
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

    #[test]
    fn test_market_order_creation_with_price_update() {
        let market_id = get_random_uuid();
        let mut market_book = MarketBook::new(dec!(100)); // LMSR with b=100

        // Add initial liquidity on both sides
        let buy_order_yes = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.45),
            quantity: dec!(20),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let sell_order_yes = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.55),
            quantity: dec!(15),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let buy_order_no = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::NO,
            price: dec!(0.40),
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        // Add initial orders to establish baseline
        market_book.add_order(&buy_order_yes);
        market_book.add_order(&sell_order_yes);
        market_book.add_order(&buy_order_no);

        let initial_yes_price = market_book.current_yes_price;
        let initial_no_price = market_book.current_no_price;

        println!("Initial YES price: {}", initial_yes_price);
        println!("Initial NO price: {}", initial_no_price);

        // Create a large market buy order for YES outcome
        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),    // Market order starts with zero price
            quantity: dec!(0), // Will be calculated based on budget
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::MARKET,
        };

        let budget = dec!(10); // $10 budget for market order
        let matches = market_book.create_market_order(&mut market_buy_order, budget);

        // Verify market order execution
        assert!(
            matches.len() > 0,
            "Market order should have matched with existing orders"
        );
        assert_eq!(
            market_buy_order.price,
            dec!(0),
            "Market order should maintain zero price"
        );
        assert!(
            market_buy_order.quantity > dec!(0),
            "Market order should have calculated quantity"
        );

        // Verify price impact
        let new_yes_price = market_book.current_yes_price;
        let new_no_price = market_book.current_no_price;

        println!("After market buy YES:");
        println!("New YES price: {}", new_yes_price);
        println!("New NO price: {}", new_no_price);

        // YES price should increase due to increased buying pressure
        assert!(
            new_yes_price > initial_yes_price,
            "YES price should increase after large market buy"
        );
        assert!(
            new_no_price < initial_no_price,
            "NO price should decrease correspondingly"
        );

        // Prices should still sum to approximately 1 (within reasonable tolerance)
        let price_sum = new_yes_price + new_no_price;
        assert!(
            (price_sum - dec!(1)).abs() < dec!(0.01),
            "Prices should sum to approximately 1"
        );

        // Now create a market sell order for YES to test opposite direction
        let mut market_sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),
            quantity: dec!(0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::MARKET,
        };

        let sell_budget = dec!(8); // $8 target revenue
        market_book.create_market_order(&mut market_sell_order, sell_budget);

        let final_yes_price = market_book.current_yes_price;
        let final_no_price = market_book.current_no_price;

        println!("After market sell YES:");
        println!("Final YES price: {}", final_yes_price);
        println!("Final NO price: {}", final_no_price);

        // YES price should decrease due to selling pressure
        assert!(
            final_yes_price < new_yes_price,
            "YES price should decrease after market sell"
        );
        assert!(
            final_no_price > new_no_price,
            "NO price should increase correspondingly"
        );
    }

    #[test]
    fn test_market_order_with_zero_liquidity_price_update() {
        let market_id = get_random_uuid();
        let mut market_book = MarketBook::new(dec!(0)); // No LMSR liquidity, uses midpoint pricing

        // Add orders to create spread
        let buy_order_yes = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.40), // Bid
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let sell_order_yes = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.60), // Ask
            quantity: dec!(10),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        market_book.add_order(&buy_order_yes);
        market_book.add_order(&sell_order_yes);

        let initial_yes_price = market_book.current_yes_price;
        println!("Initial YES price (midpoint): {}", initial_yes_price);

        // Should be midpoint of 0.40 and 0.60 = 0.50
        assert_eq!(initial_yes_price, dec!(0.50), "Should use midpoint pricing");

        // Create market order that consumes the ask
        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),
            quantity: dec!(0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::MARKET,
        };

        let budget = dec!(6); // Enough to buy all 10 shares at 0.60
        let matches = market_book.create_market_order(&mut market_buy_order, budget);

        assert!(matches.len() > 0, "Should match with sell orders");

        let new_yes_price = market_book.current_yes_price;
        println!("New YES price after consuming ask: {}", new_yes_price);

        // After consuming the ask, only bid should remain, so price should be capped
        assert!(
            new_yes_price <= dec!(0.95),
            "Price should be capped at 0.95 when only bid exists"
        );
        assert!(
            new_yes_price >= dec!(0.40),
            "Price should be at least the remaining bid price"
        );
    }

    #[test]
    fn test_market_order_large_impact_on_thin_book() {
        let market_id = get_random_uuid();
        let mut market_book = MarketBook::new(dec!(50)); // Moderate LMSR liquidity

        // Add minimal liquidity
        let small_sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.50),
            quantity: dec!(2), // Very small quantity
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        market_book.add_order(&small_sell_order);

        let initial_yes_price = market_book.current_yes_price;
        println!("Initial YES price with thin book: {}", initial_yes_price);

        // Large market order relative to available liquidity
        let mut large_market_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),
            quantity: dec!(0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::MARKET,
        };

        let large_budget = dec!(50); // Much larger than available liquidity
        let matches = market_book.create_market_order(&mut large_market_order, large_budget);

        let final_yes_price = market_book.current_yes_price;
        println!("Final YES price after large order: {}", final_yes_price);

        // Should have some price impact even with thin book due to LMSR
        assert!(
            final_yes_price != initial_yes_price,
            "Price should change with market activity"
        );

        // Verify market order was partially filled
        assert!(
            large_market_order.quantity <= dec!(2),
            "Should only fill available quantity"
        );
        assert_eq!(
            matches.len(),
            1,
            "Should match with the one available order"
        );
    }

    #[test]
    fn test_market_order_cross_outcome_price_consistency() {
        let market_id = get_random_uuid();
        let mut market_book = MarketBook::new(dec!(100));

        // Add balanced liquidity to both outcomes
        let buy_yes = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.48),
            quantity: dec!(20),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let buy_no = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::NO,
            price: dec!(0.52),
            quantity: dec!(20),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        market_book.add_order(&buy_yes);
        market_book.add_order(&buy_no);

        let initial_sum = market_book.current_yes_price + market_book.current_no_price;
        println!("Initial price sum: {}", initial_sum);

        // Large market buy for YES
        let mut market_order_yes = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),
            quantity: dec!(0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::MARKET,
        };

        // Add sell orders for the market order to match against
        let sell_yes = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.52),
            quantity: dec!(30),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        market_book.add_order(&sell_yes);

        let budget = dec!(15);
        market_book.create_market_order(&mut market_order_yes, budget);

        let final_sum = market_book.current_yes_price + market_book.current_no_price;
        println!("Final price sum: {}", final_sum);
        println!(
            "YES: {}, NO: {}",
            market_book.current_yes_price, market_book.current_no_price
        );

        // Prices should still be consistent (sum to ~1)
        assert!(
            (final_sum - dec!(1)).abs() < dec!(0.05),
            "Price sum should remain close to 1"
        );

        // YES price should have increased due to buying pressure
        assert!(
            market_book.current_yes_price > dec!(0.5),
            "YES price should be above 0.5 after buying"
        );
        assert!(
            market_book.current_no_price < dec!(0.5),
            "NO price should be below 0.5 correspondingly"
        );
    }
}
