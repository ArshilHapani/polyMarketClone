use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDateTime;
use db_service::schema::{
    enums::{OrderSide, OrderStatus, Outcome},
    orders::Order,
};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct GlobalOrderBook {
    pub(crate) markets: HashMap<Uuid, MarketOrderBook>,
}

#[derive(Debug, Default)]
pub(crate) struct MarketOrderBook {
    yes_book: OutcomeOrderBook,
    no_book: OutcomeOrderBook,
    pub(crate) market_id: Uuid,
    pub(crate) current_yes_price: Decimal,
    pub(crate) current_no_price: Decimal,
    pub(crate) liquidity_b: Decimal,
}

#[derive(Debug, Default)]
pub(crate) struct OutcomeOrderBook {
    pub(crate) bids: BTreeMap<Decimal, PriceLevel>, // sellers
    pub(crate) asks: BTreeMap<Decimal, PriceLevel>, // buyers
}

#[derive(Debug, Default)]
pub(crate) struct PriceLevel {
    orders: Vec<OrderBookEntry>,
    pub(crate) total_quantity: Decimal,
}

#[derive(Debug, Clone)]
struct OrderBookEntry {
    pub(crate) order_id: Uuid,
    pub(crate) user_id: Uuid,
    /// Total quantity of the order
    pub(crate) quantity: Decimal,
    /// Filled quantity of the order
    pub(crate) filled_quantity: Decimal,
    pub(crate) timestamp: NaiveDateTime,
}

impl OutcomeOrderBook {
    pub(crate) fn add_order(&mut self, order: &Order) {
        let price_map = match order.side {
            OrderSide::BUY => &mut self.bids,
            OrderSide::SELL => &mut self.asks,
        };

        let price_level = price_map.entry(order.price).or_default();

        let entry = OrderBookEntry {
            order_id: order.id,
            user_id: order.user_id,
            quantity: order.quantity,
            filled_quantity: order.filled_quantity,
            timestamp: order.created_at,
        };

        price_level.orders.push(entry);
        price_level.total_quantity += order.quantity - order.filled_quantity;
    }

    pub(crate) fn best_bid(&self) -> Option<Decimal> {
        // sorted in descending order, so we take the last one (highest)
        self.bids.keys().next_back().cloned()
    }

    pub(crate) fn best_ask(&self) -> Option<Decimal> {
        // sorted in ascending order, so we take the first one (cheapest)
        self.asks.keys().next().cloned()
    }

    pub(crate) fn remove_order(&mut self, order_id: Uuid, side: OrderSide, price: Decimal) -> bool {
        let price_map = match side {
            OrderSide::BUY => &mut self.bids,
            OrderSide::SELL => &mut self.asks,
        };

        if let Some(price_level) = price_map.get_mut(&price) {
            if let Some(pos) = price_level
                .orders
                .iter()
                .position(|o| o.order_id == order_id)
            {
                let removed_order = price_level.orders.remove(pos);
                price_level.total_quantity -=
                    removed_order.quantity - removed_order.filled_quantity;

                // if there are no orders left at this price level, remove it
                if price_level.orders.is_empty() {
                    price_map.remove(&price);
                }

                return true;
            }
        }
        false
    }

    pub(crate) fn update_order(
        &mut self,
        order_id: Uuid,
        side: OrderSide,
        price: Decimal,
        new_filled_quantity: Decimal,
    ) -> bool {
        let price_map = match side {
            OrderSide::BUY => &mut self.bids,
            OrderSide::SELL => &mut self.asks,
        };

        if let Some(price_level) = price_map.get_mut(&price) {
            if let Some(order) = price_level
                .orders
                .iter_mut()
                .find(|o| o.order_id == order_id)
            {
                let prev_remaining = order.quantity - order.filled_quantity;
                let new_remaining = order.quantity - new_filled_quantity;
                price_level.total_quantity =
                    price_level.total_quantity + new_remaining - prev_remaining;

                order.filled_quantity = new_filled_quantity;

                if price_level.total_quantity <= Decimal::ZERO {
                    price_map.remove(&price);
                }

                return true;
            }
        }
        false
    }

    pub(crate) fn match_order(&mut self, order: &mut Order) -> Vec<(Uuid, Uuid, Decimal, Decimal)> {
        // order id, opposite order id, quantity matched, price
        let mut matches: Vec<(Uuid, Uuid, Decimal, Decimal)> = Vec::new();

        let (book, is_buy) = match order.side {
            OrderSide::BUY => (&mut self.asks, true),
            OrderSide::SELL => (&mut self.bids, false),
        };

        let mut keys: Vec<Decimal> = book.keys().cloned().collect();
        if is_buy {
            // sort in ascending order
            keys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            // sort in descending order
            keys.sort_by(|a, b| b.partial_cmp(a).unwrap());
        }

        let mut remaining = order.quantity - order.filled_quantity; // if order is already filled
        if remaining <= Decimal::ZERO {
            return matches;
        }

        for price in keys {
            // order price boundary check
            if (is_buy && order.price < price) || (!is_buy && order.price > price) {
                continue;
            }

            if let Some(price_level) = book.get_mut(&price) {
                let mut new_orders = Vec::new(); // FIXME: creating new vector every time... instead of internal vector mutation
                for mut opposite_order in price_level.orders.drain(..) {
                    let opp_remaining = opposite_order.quantity - opposite_order.filled_quantity;
                    if opp_remaining <= Decimal::ZERO {
                        continue;
                    }

                    let match_qty = remaining.min(opp_remaining);

                    opposite_order.filled_quantity += match_qty;
                    order.filled_quantity += match_qty;
                    remaining -= match_qty;

                    matches.push((order.id, opposite_order.order_id, match_qty, price));

                    if opposite_order.filled_quantity < opposite_order.quantity {
                        new_orders.push(opposite_order);
                    }

                    if remaining == Decimal::ZERO {
                        break;
                    }
                }
                price_level.orders = new_orders;
                price_level.total_quantity = price_level
                    .orders
                    .iter()
                    .map(|o| o.quantity - o.filled_quantity)
                    .sum();

                if price_level.orders.is_empty() {
                    book.remove(&price);
                }

                if remaining == Decimal::ZERO {
                    break;
                }
            }
        }

        if order.filled_quantity == order.quantity {
            order.status = OrderStatus::FILLED;
        }

        matches
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_order(
        id: Uuid,
        user_id: Uuid,
        market_id: Uuid,
        side: OrderSide,
        outcome: Outcome,
        price: Decimal,
        quantity: Decimal,
    ) -> Order {
        Order {
            id,
            user_id,
            market_id,
            side,
            outcome,
            price,
            quantity,
            filled_quantity: Decimal::ZERO,
            status: OrderStatus::OPEN,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }

    #[test]
    fn test_add_order() {
        let mut book = OutcomeOrderBook::default();
        let order_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let market_id = Uuid::new_v4();

        let order = create_test_order(
            order_id,
            user_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(80, 2), // 0.80
            Decimal::new(10, 0), // 10
        );

        book.add_order(&order);

        assert_eq!(book.bids.len(), 1);
        assert_eq!(book.asks.len(), 0);

        let price_level = book.bids.get(&Decimal::new(80, 2)).unwrap();
        assert_eq!(price_level.total_quantity, Decimal::new(10, 0));
        assert_eq!(price_level.orders.len(), 1);
        assert_eq!(price_level.orders[0].order_id, order_id);
    }

    #[test]
    fn test_best_bid_ask() {
        let mut book = OutcomeOrderBook::default();
        let user_id = Uuid::new_v4();
        let market_id = Uuid::new_v4();

        // Add buy orders at different prices
        let buy_order1 = create_test_order(
            Uuid::new_v4(),
            user_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(80, 2), // 0.80
            Decimal::new(5, 0),  // 5
        );

        let buy_order2 = create_test_order(
            Uuid::new_v4(),
            user_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(85, 2), // 0.85
            Decimal::new(3, 0),  // 3
        );

        // Add sell orders at different prices
        let sell_order1 = create_test_order(
            Uuid::new_v4(),
            user_id,
            market_id,
            OrderSide::SELL,
            Outcome::YES,
            Decimal::new(90, 2), // 0.90
            Decimal::new(4, 0),  // 4
        );

        let sell_order2 = create_test_order(
            Uuid::new_v4(),
            user_id,
            market_id,
            OrderSide::SELL,
            Outcome::YES,
            Decimal::new(95, 2), // 0.95
            Decimal::new(2, 0),  // 2
        );

        book.add_order(&buy_order1);
        book.add_order(&buy_order2);
        book.add_order(&sell_order1);
        book.add_order(&sell_order2);

        assert_eq!(book.best_bid(), Some(Decimal::new(85, 2))); // Highest buy price
        assert_eq!(book.best_ask(), Some(Decimal::new(90, 2))); // Lowest sell price
    }

    #[test]
    fn test_remove_order() {
        let mut book = OutcomeOrderBook::default();
        let user_id = Uuid::new_v4();
        let market_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();

        let order = create_test_order(
            order_id,
            user_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(80, 2), // 0.80
            Decimal::new(10, 0), // 10
        );

        book.add_order(&order);
        assert_eq!(book.bids.len(), 1);

        // Remove the order
        let result = book.remove_order(order_id, OrderSide::BUY, Decimal::new(80, 2));
        assert!(result);
        assert_eq!(book.bids.len(), 0);

        // Try to remove non-existent order
        let result = book.remove_order(Uuid::new_v4(), OrderSide::BUY, Decimal::new(80, 2));
        assert!(!result);
    }

    #[test]
    fn test_update_order() {
        let mut book = OutcomeOrderBook::default();
        let user_id = Uuid::new_v4();
        let market_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();

        let order = create_test_order(
            order_id,
            user_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(80, 2), // 0.80
            Decimal::new(10, 0), // 10
        );

        book.add_order(&order);

        // Update with partial fill
        let result = book.update_order(
            order_id,
            OrderSide::BUY,
            Decimal::new(80, 2),
            Decimal::new(5, 0),
        );
        assert!(result);

        let price_level = book.bids.get(&Decimal::new(80, 2)).unwrap();
        assert_eq!(price_level.total_quantity, Decimal::new(5, 0)); // 10 - 5 = 5 remaining

        // Update with full fill (should remove the order)
        let result = book.update_order(
            order_id,
            OrderSide::BUY,
            Decimal::new(80, 2),
            Decimal::new(10, 0),
        );
        assert!(result);
        assert_eq!(book.bids.len(), 0);
    }

    #[test]
    fn test_match_order() {
        let mut book = OutcomeOrderBook::default();
        let user_id = Uuid::new_v4();
        let market_id = Uuid::new_v4();

        // Add a sell order
        let sell_order_id = Uuid::new_v4();
        let sell_order = create_test_order(
            sell_order_id,
            user_id,
            market_id,
            OrderSide::SELL,
            Outcome::YES,
            Decimal::new(80, 2), // 0.80
            Decimal::new(10, 0), // 10
        );

        book.add_order(&sell_order);

        // Create a matching buy order
        let buy_order_id = Uuid::new_v4();
        let mut buy_order = create_test_order(
            buy_order_id,
            user_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(85, 2), // 0.85 (willing to pay more than ask)
            Decimal::new(5, 0),  // 5
        );

        // Match the order
        let matches = book.match_order(&mut buy_order);

        // Verify matches
        assert_eq!(matches.len(), 1);
        let (matched_buy_id, matched_sell_id, matched_qty, matched_price) = matches[0];
        assert_eq!(matched_buy_id, buy_order_id);
        assert_eq!(matched_sell_id, sell_order_id);
        assert_eq!(matched_qty, Decimal::new(5, 0));
        assert_eq!(matched_price, Decimal::new(80, 2));

        // Verify order book state
        assert_eq!(book.asks.len(), 1);
        let price_level = book.asks.get(&Decimal::new(80, 2)).unwrap();
        assert_eq!(price_level.total_quantity, Decimal::new(5, 0)); // 10 - 5 = 5 remaining
    }

    #[test]
    fn test_market_order_book() {
        let market_id = Uuid::new_v4();
        let mut market_book = MarketOrderBook::new(market_id, Decimal::ZERO);
        let user_id = Uuid::new_v4();

        // Add YES orders
        let yes_buy_order = create_test_order(
            Uuid::new_v4(),
            user_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(80, 2), // 0.80
            Decimal::new(10, 0), // 10
        );

        let yes_sell_order = create_test_order(
            Uuid::new_v4(),
            user_id,
            market_id,
            OrderSide::SELL,
            Outcome::YES,
            Decimal::new(90, 2), // 0.90
            Decimal::new(8, 0),  // 8
        );

        // Add NO orders
        let no_buy_order = create_test_order(
            Uuid::new_v4(),
            user_id,
            market_id,
            OrderSide::BUY,
            Outcome::NO,
            Decimal::new(70, 2), // 0.70
            Decimal::new(6, 0),  // 6
        );

        let no_sell_order = create_test_order(
            Uuid::new_v4(),
            user_id,
            market_id,
            OrderSide::SELL,
            Outcome::NO,
            Decimal::new(75, 2), // 0.75
            Decimal::new(4, 0),  // 4
        );

        market_book.add_order(&yes_buy_order);
        market_book.add_order(&yes_sell_order);
        market_book.add_order(&no_buy_order);
        market_book.add_order(&no_sell_order);

        // Test order book access
        assert!(market_book.get_order_book(&Outcome::YES).is_some());
        assert!(market_book.get_order_book(&Outcome::NO).is_some());
        assert!(market_book.get_order_book(&Outcome::UNSPECIFIED).is_none());

        // Test price calculation
        assert_eq!(market_book.current_yes_price, Decimal::new(85, 2)); // (0.80 + 0.90) / 2
        assert_eq!(market_book.current_no_price, Decimal::new(725, 3)); // (0.70 + 0.75) / 2
    }

    #[test]
    fn test_global_order_book() {
        let mut global_book = GlobalOrderBook::new();
        let market_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Create and process an order
        let mut order = create_test_order(
            Uuid::new_v4(),
            user_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(80, 2), // 0.80
            Decimal::new(10, 0), // 10
        );

        global_book.process_order(&mut order, Decimal::ZERO);

        // Test market retrieval
        assert!(global_book.markets.contains_key(&market_id));

        // Test price retrieval
        let yes_price = global_book.get_market_price(&market_id, Outcome::YES);
        assert!(yes_price.is_some());
        assert_eq!(yes_price.unwrap(), Decimal::new(5, 1)); // Default price when only one side has orders
    }

    #[test]
    fn test_liquidity_based_pricing() {
        let market_id = Uuid::new_v4();
        let liquidity_b = Decimal::new(1000, 0); // 1000 units of liquidity
        let mut market_book = MarketOrderBook::new(market_id, liquidity_b);
        let user_id = Uuid::new_v4();

        // Add YES buy orders (bids)
        let yes_buy_order = create_test_order(
            Uuid::new_v4(),
            user_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(80, 2),  // 0.80
            Decimal::new(200, 0), // 200
        );

        // Add NO buy orders (bids)
        let no_buy_order = create_test_order(
            Uuid::new_v4(),
            user_id,
            market_id,
            OrderSide::BUY,
            Outcome::NO,
            Decimal::new(70, 2),  // 0.70
            Decimal::new(300, 0), // 300
        );

        market_book.add_order(&yes_buy_order);
        market_book.add_order(&no_buy_order);

        // Calculate expected prices using LMSR formula
        // YES price = liquidity_b / (liquidity_b + funds_no)
        // NO price = liquidity_b / (liquidity_b + funds_yes)
        let funds_yes = Decimal::new(80, 2) * Decimal::new(200, 0); // 0.80 * 200 = 160
        let funds_no = Decimal::new(70, 2) * Decimal::new(300, 0); // 0.70 * 300 = 210

        let expected_yes_price = liquidity_b / (liquidity_b + funds_no);
        let expected_no_price = liquidity_b / (liquidity_b + funds_yes);

        assert_eq!(market_book.current_yes_price, expected_yes_price);
        assert_eq!(market_book.current_no_price, expected_no_price);
    }

    #[test]
    fn test_full_order_matching_flow() {
        let mut global_book = GlobalOrderBook::new();
        let market_id = Uuid::new_v4();
        let user1_id = Uuid::new_v4();
        let user2_id = Uuid::new_v4();

        // User 1 places a sell order for YES outcome
        let sell_order_id = Uuid::new_v4();
        let mut sell_order = create_test_order(
            sell_order_id,
            user1_id,
            market_id,
            OrderSide::SELL,
            Outcome::YES,
            Decimal::new(80, 2), // 0.80
            Decimal::new(10, 0), // 10
        );

        global_book.process_order(&mut sell_order, Decimal::ZERO);
        assert_eq!(sell_order.status, OrderStatus::OPEN);

        // User 2 places a buy order for YES outcome that matches
        let buy_order_id = Uuid::new_v4();
        let mut buy_order = create_test_order(
            buy_order_id,
            user2_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(85, 2), // 0.85 (willing to pay more than ask)
            Decimal::new(10, 0), // 10
        );

        let matches = global_book.process_order(&mut buy_order, Decimal::ZERO);

        // Verify matches
        assert_eq!(matches.len(), 1);
        let (matched_buy_id, matched_sell_id, matched_qty, matched_price) = matches[0];
        assert_eq!(matched_buy_id, buy_order_id);
        assert_eq!(matched_sell_id, sell_order_id);
        assert_eq!(matched_qty, Decimal::new(10, 0));
        assert_eq!(matched_price, Decimal::new(80, 2));

        // Verify order statuses
        assert_eq!(buy_order.status, OrderStatus::FILLED);
        assert_eq!(buy_order.filled_quantity, Decimal::new(10, 0));
    }

    #[test]
    fn test_edge_case_zero_quantity_order() {
        let mut book = OutcomeOrderBook::default();
        let user_id = Uuid::new_v4();
        let market_id = Uuid::new_v4();

        // Create an order with zero quantity
        let order = create_test_order(
            Uuid::new_v4(),
            user_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(80, 2), // 0.80
            Decimal::ZERO,       // 0 quantity
        );

        book.add_order(&order);

        // The order should be added but with zero remaining quantity
        assert_eq!(book.bids.len(), 1);
        let price_level = book.bids.get(&Decimal::new(80, 2)).unwrap();
        assert_eq!(price_level.total_quantity, Decimal::ZERO);
    }

    #[test]
    fn test_large_quantity_order_matching() {
        let mut book = OutcomeOrderBook::default();
        let market_id = Uuid::new_v4();
        let user1_id = Uuid::new_v4();
        let user2_id = Uuid::new_v4();

        // Add a sell order with large quantity
        let sell_order_id = Uuid::new_v4();
        let sell_order = create_test_order(
            sell_order_id,
            user1_id,
            market_id,
            OrderSide::SELL,
            Outcome::YES,
            Decimal::new(80, 2),   // 0.80
            Decimal::new(5000, 0), // 5000 units
        );

        book.add_order(&sell_order);

        // Create a matching buy order with large quantity
        let buy_order_id = Uuid::new_v4();
        let mut buy_order = create_test_order(
            buy_order_id,
            user2_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(85, 2),   // 0.85
            Decimal::new(3000, 0), // 3000 units
        );

        // Match the order
        let matches = book.match_order(&mut buy_order);

        // Verify matches
        assert_eq!(matches.len(), 1);
        let (_matched_buy_id, _matched_sell_id, matched_qty, _matched_price) = matches[0];
        assert_eq!(matched_qty, Decimal::new(3000, 0));
        assert_eq!(buy_order.filled_quantity, Decimal::new(3000, 0));

        // Verify remaining sell order
        assert_eq!(book.asks.len(), 1);
        let price_level = book.asks.get(&Decimal::new(80, 2)).unwrap();
        assert_eq!(price_level.total_quantity, Decimal::new(2000, 0)); // 5000 - 3000 = 2000
    }

    #[test]
    fn test_multiple_partial_fills() {
        let mut book = OutcomeOrderBook::default();
        let market_id = Uuid::new_v4();
        let seller_id = Uuid::new_v4();
        let buyer_id = Uuid::new_v4();

        // Add multiple sell orders at different price levels
        let sell_orders = [
            create_test_order(
                Uuid::new_v4(),
                seller_id,
                market_id,
                OrderSide::SELL,
                Outcome::YES,
                Decimal::new(80, 2),  // 0.80
                Decimal::new(200, 0), // 200
            ),
            create_test_order(
                Uuid::new_v4(),
                seller_id,
                market_id,
                OrderSide::SELL,
                Outcome::YES,
                Decimal::new(85, 2),  // 0.85
                Decimal::new(300, 0), // 300
            ),
            create_test_order(
                Uuid::new_v4(),
                seller_id,
                market_id,
                OrderSide::SELL,
                Outcome::YES,
                Decimal::new(90, 2),  // 0.90
                Decimal::new(500, 0), // 500
            ),
        ];

        for order in &sell_orders {
            book.add_order(order);
        }

        // Create a large buy order that should match against all sell orders
        let mut buy_order = create_test_order(
            Uuid::new_v4(),
            buyer_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(95, 2),   // 0.95 (willing to pay more than all asks)
            Decimal::new(1000, 0), // 1000 (matches all 200+300+500)
        );

        // Match the order
        let matches = book.match_order(&mut buy_order);

        // Verify matches
        assert_eq!(matches.len(), 3); // Should match with all 3 sell orders
        assert_eq!(buy_order.filled_quantity, Decimal::new(1000, 0));
        assert_eq!(buy_order.status, OrderStatus::FILLED);

        // Order book should be empty
        assert_eq!(book.asks.len(), 0);
    }

    #[test]
    fn test_exact_price_matching() {
        let mut book = OutcomeOrderBook::default();
        let market_id = Uuid::new_v4();
        let user1_id = Uuid::new_v4();
        let user2_id = Uuid::new_v4();

        // Add a sell order
        let sell_order_id = Uuid::new_v4();
        let sell_order = create_test_order(
            sell_order_id,
            user1_id,
            market_id,
            OrderSide::SELL,
            Outcome::YES,
            Decimal::new(80, 2), // 0.80
            Decimal::new(10, 0), // 10
        );

        book.add_order(&sell_order);

        // Create a buy order with exactly the same price
        let buy_order_id = Uuid::new_v4();
        let mut buy_order = create_test_order(
            buy_order_id,
            user2_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(80, 2), // 0.80 exactly
            Decimal::new(5, 0),  // 5
        );

        // Match the order
        let matches = book.match_order(&mut buy_order);

        // Verify matches - should match since buy price >= sell price
        assert_eq!(matches.len(), 1);
        let (_matched_buy_id, _matched_sell_id, matched_qty, matched_price) = matches[0];
        assert_eq!(matched_qty, Decimal::new(5, 0));
        assert_eq!(matched_price, Decimal::new(80, 2));
    }

    #[test]
    fn test_multiple_market_interactions() {
        let mut global_book = GlobalOrderBook::new();
        let market1_id = Uuid::new_v4();
        let market2_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Add orders to first market
        let mut order1 = create_test_order(
            Uuid::new_v4(),
            user_id,
            market1_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(80, 2),   // 0.80
            Decimal::new(1000, 0), // 1000
        );

        global_book.process_order(&mut order1, Decimal::new(500, 0));

        // Add orders to second market
        let mut order2 = create_test_order(
            Uuid::new_v4(),
            user_id,
            market2_id,
            OrderSide::BUY,
            Outcome::NO,
            Decimal::new(70, 2),   // 0.70
            Decimal::new(2000, 0), // 2000
        );

        global_book.process_order(&mut order2, Decimal::new(1000, 0));

        // Verify both markets exist
        assert_eq!(global_book.markets.len(), 2);
        assert!(global_book.markets.contains_key(&market1_id));
        assert!(global_book.markets.contains_key(&market2_id));

        // Verify each market has the correct liquidity parameter
        assert_eq!(
            global_book.markets.get(&market1_id).unwrap().liquidity_b,
            Decimal::new(500, 0)
        );
        assert_eq!(
            global_book.markets.get(&market2_id).unwrap().liquidity_b,
            Decimal::new(1000, 0)
        );
    }

    #[test]
    fn test_decimal_precision_edge_cases() {
        let mut book = OutcomeOrderBook::default();
        let market_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Add a sell order with many decimal places
        let sell_order = create_test_order(
            Uuid::new_v4(),
            user_id,
            market_id,
            OrderSide::SELL,
            Outcome::YES,
            Decimal::new(12345, 5), // 0.12345
            Decimal::new(10, 0),    // 10
        );

        book.add_order(&sell_order);

        // Create a buy order with slightly higher price
        let mut buy_order = create_test_order(
            Uuid::new_v4(),
            user_id,
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(12346, 5), // 0.12346
            Decimal::new(5, 0),     // 5
        );

        // Match the order
        let matches = book.match_order(&mut buy_order);

        // Verify matches
        assert_eq!(matches.len(), 1);
        let (_, _, matched_qty, matched_price) = matches[0];
        assert_eq!(matched_qty, Decimal::new(5, 0));
        assert_eq!(matched_price, Decimal::new(12345, 5)); // Should match at the sell price
    }

    #[test]
    fn process_multi_user_order() {
        let mut global_book = GlobalOrderBook::new();
        let market_id = Uuid::new_v4();

        // YES outcome buy order
        let mut order_1 = create_test_order(
            Uuid::new_v4(),
            Uuid::new_v4(),
            market_id,
            OrderSide::BUY,
            Outcome::YES,
            Decimal::new(25, 1), // 0.25
            Decimal::new(10, 0), // 10
        );

        // YES outcome sell order
        let mut order_2 = create_test_order(
            Uuid::new_v4(),
            Uuid::new_v4(),
            market_id,
            OrderSide::SELL,
            Outcome::YES,
            Decimal::new(30, 1), // 0.20
            Decimal::new(9, 0),  // 5
        );

        global_book.process_order(&mut order_1, Decimal::ZERO);

        // println!("Global order book 1 {:#?}", global_book);

        global_book.process_order(&mut order_2, Decimal::ZERO);

        // println!("Global order book 2 {:#?}", global_book);
        assert!(true);
    }
}
