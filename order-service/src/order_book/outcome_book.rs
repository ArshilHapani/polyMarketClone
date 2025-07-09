/*
 * This calculation is based on 0.0 decimal precision.
 * Means 0.3 is 0.3 here, not 30. That thing must be handled by above level data processing
 *
 * Price for every incoming order must be between 0 to 1 (inclusive).
 */

use std::collections::BTreeMap;

use db_service::schema::{
    enums::{OrderSide, OrderStatus},
    orders::Order,
};
use rust_decimal::Decimal;
use utility_helpers::{
    log_info,
    types::{OrderBookDataStruct, OrderLevel},
};
use uuid::Uuid;

#[derive(Default, Debug)]
pub(crate) struct PriceLevel {
    pub(crate) orders: Vec<OrderBookEntry>, // should I consider using hashmap here for O(1) lookup
    pub(crate) total_quantity: Decimal,
}

#[derive(Debug)]
pub(crate) struct OrderBookEntry {
    pub user_id: Uuid,
    pub order_id: Uuid,
    pub total_quantity: Decimal,
    pub filled_quantity: Decimal,
}

#[derive(Debug, Default)]
pub(crate) struct OutcomeBook {
    pub(crate) bids: BTreeMap<Decimal, PriceLevel>, // buyers side
    pub(crate) asks: BTreeMap<Decimal, PriceLevel>, // sellers side
}

#[derive(Debug)]
pub(crate) struct OrderBookMatchedOutput {
    pub order_id: Uuid,
    pub opposite_order_id: Uuid,
    pub matched_quantity: Decimal,
    pub price: Decimal,
    pub opposite_order_total_quantity: Decimal,
    pub opposite_order_filled_quantity: Decimal,
}

impl OutcomeBook {
    pub(crate) fn add_order(&mut self, order: &Order) {
        if order.price > Decimal::ONE {
            log_info!(
                "Order price should be less than or equal to 1.0, but got: {}, not adding order",
                order.price
            );
            return; // price should be less than or equal to 1.0 (or 100%)
        }
        let side = match order.side {
            OrderSide::BUY => &mut self.bids,
            OrderSide::SELL => &mut self.asks,
        };

        let price_level = side.entry(order.price).or_default();

        let entry = OrderBookEntry {
            filled_quantity: order.filled_quantity,
            order_id: order.id,
            total_quantity: order.quantity,
            user_id: order.user_id,
        };

        price_level.orders.push(entry);
        price_level.total_quantity += order.quantity - order.filled_quantity;
    }

    pub(super) fn best_bid(&self) -> Option<Decimal> {
        // sorted in ascending order, so we take the last one (highest available price from buyers to sellers)
        self.bids.keys().next_back().cloned()
    }

    pub(super) fn best_ask(&self) -> Option<Decimal> {
        // keys are sorted in ascending order, so lowest price from sellers to buyers is first
        self.asks.keys().next().cloned()
    }

    pub(super) fn remove_order(&mut self, order_id: Uuid, side: OrderSide, price: Decimal) -> bool {
        if price > Decimal::ONE {
            log_info!(
                "Order price should be less than or equal to 1.0, but got: {}",
                price
            );
            return false; // price should be less than or equal to 1.0 (or 100%)
        }
        let price_side = match side {
            OrderSide::BUY => &mut self.bids,
            OrderSide::SELL => &mut self.asks,
        };
        if let Some(price_level) = price_side.get_mut(&price) {
            if let Some(pos) = price_level
                .orders
                .iter()
                .position(|order| order.order_id == order_id)
            {
                let removed_order = price_level.orders.remove(pos);
                price_level.total_quantity -=
                    removed_order.total_quantity - removed_order.filled_quantity;

                if price_level.orders.is_empty() {
                    price_side.remove(&price);
                }
                return true;
            }
        }
        false
    }

    // returns matched orders if updated order is matched with some order
    pub(super) fn update_order(
        &mut self,
        order: &mut Order,
        updated_price: Decimal,
        new_quantity: Decimal,
    ) -> bool {
        if order.price > Decimal::ONE {
            log_info!(
                "Order price should be less than or equal to 1.0, but got: {}",
                order.price
            );
            return false; // invalid price
        }
        if order.quantity == new_quantity && order.price == updated_price {
            log_info!("No changes in order, nothing to update");
            return true; // no changes
        }
        // removing order
        if !self.remove_order(order.id, order.side, order.price) {
            log_info!("Order not found in book, cannot update");
            return false; // order not found
        }
        order.price = updated_price;
        order.quantity = new_quantity;
        order.status = OrderStatus::OPEN; // resetting status to open

        self.add_order(order);

        true
    }

    pub(super) fn _update_order_filled_quantity(
        &mut self,
        order_id: Uuid,
        side: OrderSide,
        current_price: Decimal,
        new_filled_quantity: Decimal,
    ) -> bool {
        let price_mapping = match side {
            OrderSide::BUY => &mut self.bids,
            OrderSide::SELL => &mut self.asks,
        };
        if let Some(price_level) = price_mapping.get_mut(&current_price) {
            if let Some(order) = price_level
                .orders
                .iter_mut()
                .find(|order| order.order_id == order_id)
            {
                /*
                   35 price_level.total_quantity

                   10 order.total_quantity (already exists)
                   5 order.filled_quantity
                   5 order.remaining_quantity

                   price_level.total_quantity = 30 (30 - 5)

                   update
                   10 -> order.total_quantity
                   5 -> order.filled_quantity
                   7 -> new_filled_quantity
                   prev_remaining = 10 - 5 = 5
                   new_remaining = 10 - 7 = 3

                   price_level.total_quantity = 30 + 3 - 5 = 28
                */
                let prev_remaining = order.total_quantity - order.filled_quantity;

                let new_remaining = order.total_quantity - new_filled_quantity;

                price_level.total_quantity =
                    price_level.total_quantity + new_remaining - prev_remaining;
                order.filled_quantity = new_filled_quantity;

                if price_level.total_quantity <= Decimal::ZERO {
                    price_mapping.remove(&current_price);
                }

                return true;
            }
        }

        false
    }

    pub(super) fn match_order(&mut self, order: &mut Order) -> Vec<OrderBookMatchedOutput> {
        // order id, opposite order id, matched quantity, price
        let mut matches: Vec<OrderBookMatchedOutput> = Vec::new();

        if order.status != OrderStatus::OPEN {
            return matches; // only open orders can be matched
        }
        if order.price > Decimal::ONE {
            log_info!(
                "Order price should be less than or equal to 1.0, but got: {}",
                order.price
            );
            return matches; // price should be less than or equal to 1.0 (or 100%)
        }
        if order.price != Decimal::ZERO {
            if order.quantity == Decimal::ZERO {
                order.status = OrderStatus::FILLED; // if quantity is zero, we consider it as filled
                order.filled_quantity = Decimal::ZERO; // no quantity to match

                log_info!("Order quantity is zero, nothing to match");
                return matches; // no quantity to match
            }
        }

        let (book, is_buy) = match order.side {
            OrderSide::BUY => (&mut self.asks, true), // inverse matching
            OrderSide::SELL => (&mut self.bids, false),
        };

        let mut keys: Vec<Decimal> = book.keys().cloned().collect(); // already sorted in ascending if is_buy true
        if is_buy {
            // still sorting in ascending (may be computer make mistake in case...) for buyers (best cheap price on top)
            keys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            // keys.reverse(); // TODO: research on this from asc -> desc (reversing)
            keys.sort_by(|a, b| b.partial_cmp(a).unwrap()); // sorting in descending order (for sellers best expensive price on top)
        }

        let mut remaining = order.quantity - order.filled_quantity;
        if remaining <= Decimal::ZERO {
            return matches;
        }

        for price in keys {
            // case of market order
            if order.price != Decimal::ZERO {
                if (is_buy && price > order.price) || (!is_buy && price < order.price) {
                    continue;
                }
            }

            if let Some(price_level) = book.get_mut(&price) {
                let mut orders_to_remove = Vec::new();
                for (idx, opposite_order) in price_level.orders.iter_mut().enumerate() {
                    if order.id == opposite_order.order_id
                        || order.user_id == opposite_order.user_id
                    {
                        // skip matching with itself
                        continue;
                    }
                    let opp_remaining =
                        opposite_order.total_quantity - opposite_order.filled_quantity;
                    if opp_remaining <= Decimal::ZERO {
                        continue;
                    }

                    let match_qty = remaining.min(opp_remaining);

                    ///// ATOMIC Operation START (trusting on parking lot's RWLock )
                    opposite_order.filled_quantity += match_qty;

                    order.filled_quantity += match_qty;
                    remaining -= match_qty;

                    matches.push(OrderBookMatchedOutput {
                        order_id: order.id,
                        opposite_order_id: opposite_order.order_id,
                        matched_quantity: match_qty,
                        price,
                        // price: opposite_order.price, // price of matching order
                        opposite_order_total_quantity: opposite_order.total_quantity,
                        opposite_order_filled_quantity: opposite_order.filled_quantity,
                    });

                    // pushing the index or order to remove (if filled quantity is equals to total quantity, it's because we can't borrow price_level as mutable in the current scope)
                    if opposite_order.filled_quantity == opposite_order.total_quantity {
                        orders_to_remove.push(idx);
                    }
                    if remaining == Decimal::ZERO {
                        break;
                    }
                    ///// ATOMIC Operation END
                }

                // removing orders

                price_level
                    .orders
                    .retain(|o| o.filled_quantity < o.total_quantity);

                price_level.total_quantity = price_level
                    .orders
                    .iter()
                    .map(|o| o.total_quantity - o.filled_quantity)
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

    pub(crate) fn create_market_order(
        &mut self,
        order: &mut Order,
        budget: Decimal,
    ) -> Vec<OrderBookMatchedOutput> {
        // This function is used to create a market order, which will match with the best available orders in the book
        // It will not check the price of the order, but will match with the best available orders until the quantity is filled or no more orders are available
        let order_quantity = self.get_available_match_quantity(order, budget);
        order.quantity = order_quantity; // update order quantity to the available match quantity
        order.price = Decimal::ZERO; // market orders do not have a price
        self.match_order(order)
    }

    // Getters ///

    pub(crate) fn get_order_book(&self) -> OrderBookDataStruct {
        let bids = &self.bids;
        let asks = &self.asks;

        let mut bids_values = Vec::new();
        let mut asks_values = Vec::new();

        for (price, level) in bids {
            let data = OrderLevel {
                price: *price,
                shares: level.total_quantity,
                users: level.orders.len(),
            };
            if level.orders.is_empty() {
                continue; // skip empty levels
            }
            bids_values.push(data);
        }
        for (price, level) in asks {
            let data = OrderLevel {
                price: *price,
                shares: level.total_quantity,
                users: level.orders.len(),
            };
            if level.orders.is_empty() {
                continue; // skip empty levels
            }
            asks_values.push(data);
        }

        OrderBookDataStruct {
            bids: bids_values,
            asks: asks_values,
        }
    }

    fn get_available_match_quantity(&mut self, order: &mut Order, budget: Decimal) -> Decimal {
        let mut available_quantity = Decimal::ZERO;
        if order.price != Decimal::ZERO {
            log_info!(
                "Market order price should be zero, but got: {}",
                order.price
            );
            order.price = Decimal::ZERO; // enforce market behavior
        }

        let book = match order.side {
            OrderSide::BUY => &mut self.asks,  // match against asks
            OrderSide::SELL => &mut self.bids, // match against bids
        };

        let mut keys: Vec<Decimal> = book.keys().cloned().collect();

        if order.side == OrderSide::BUY {
            keys.sort_by(|a, b| a.partial_cmp(b).unwrap()); // ascending: buy from lowest
        } else {
            keys.sort_by(|a, b| b.partial_cmp(a).unwrap()); // descending: sell to highest
        }

        let mut remaining_budget = budget;

        for price in keys {
            if remaining_budget <= Decimal::ZERO {
                break;
            }

            if let Some(level) = book.get(&price) {
                let mut total_level_qty = Decimal::ZERO;

                for entry in &level.orders {
                    if entry.user_id == order.user_id {
                        // skip matching with itself
                        continue;
                    }
                    let rem_qty = entry.total_quantity - entry.filled_quantity;
                    if rem_qty > Decimal::ZERO {
                        total_level_qty += rem_qty;
                    }
                }

                let cost_to_consume_level = price * total_level_qty;

                if remaining_budget >= cost_to_consume_level {
                    // consume full level
                    available_quantity += total_level_qty;
                    remaining_budget -= cost_to_consume_level;
                } else {
                    // partial consume
                    let partial_qty = remaining_budget / price;
                    available_quantity += partial_qty;
                    break;
                }
            }
        }

        available_quantity
    }
}

#[cfg(test)]
mod test {
    use chrono::NaiveDateTime;
    use db_service::schema::enums::{OrderType, Outcome};
    use rust_decimal_macros::dec;

    use super::*;

    fn get_created_at() -> NaiveDateTime {
        chrono::Utc::now().naive_local()
    }
    fn get_random_uuid() -> Uuid {
        Uuid::new_v4()
    }

    #[test]
    fn test_market_order_custom() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();
        let another_user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();
        let buy_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(25, 2),    // 0.25 -> 25$
            quantity: Decimal::new(10, 0), // 10
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,
            user_id,
        }; // 0.25 buy - 10 qty (buy)
        let buy_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(20, 2),   // 0.20 -> 20$
            quantity: Decimal::new(3, 0), // 3
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id,
            order_type: OrderType::LIMIT,
        }; // 0.20 buy - 3 qty (buy)

        // market order
        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::ZERO,
            quantity: Decimal::ZERO,
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: another_user_id,
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&buy_order_1);
        outcome_book.add_order(&buy_order_2);

        let budget = dec!(2.1); // equivalent to 31
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        /*
         * 0.20 * 3 = 0.60
         * 0.25 * 10 = 2.50
         * Total = 3.10
         *
         * 2.1 -> 0.20 * 3 = 0.60
         * 2.1 - 0.60 = 1.50
         * 1.50 / 0.25 = 6
         * 6 * 0.25 = 1.50
         *
         * 6 + 3 = 9
         * (above calculation is done by `get_available_match_quantity` function)
         * So, we should match 3 orders at 0.20 and 6 orders at 0.25
         */

        assert_eq!(matches.len(), 2); // should match both orders
        assert_eq!(market_buy_order.filled_quantity, dec!(9));
        assert_eq!(market_buy_order.quantity, dec!(9)); // read above comment for calculation
        assert_eq!(market_buy_order.status, OrderStatus::FILLED);
        assert_eq!(market_buy_order.price, Decimal::ZERO); // market order price is zero
    }

    #[test]
    fn test_add_order() {
        let price = Decimal::new(25, 2); // 0.25
        let created_at = get_created_at();
        let id = get_random_uuid();
        let market_id = get_random_uuid();
        let updated_at = get_created_at();
        let user_id = get_random_uuid();
        let order = Order {
            created_at,
            filled_quantity: Decimal::ZERO,
            id,
            market_id,
            outcome: Outcome::YES,         // yes side,
            price,                         // 0.25
            quantity: Decimal::new(10, 0), // 10
            side: OrderSide::BUY,
            status: OrderStatus::UNSPECIFIED,
            updated_at,
            user_id,
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();

        outcome_book.add_order(&order);

        assert_eq!(outcome_book.bids.len(), 1);

        let price_level = outcome_book.bids.get(&price).unwrap();

        assert_eq!(price_level.total_quantity, Decimal::new(10, 0));
        assert_eq!(price_level.orders.len(), 1);

        let order_book_entry = price_level.orders.get(0).unwrap();

        assert_eq!(order_book_entry.user_id, user_id);
        assert_eq!(order_book_entry.order_id, id);
        assert_eq!(order_book_entry.filled_quantity, Decimal::ZERO);
        assert_eq!(order_book_entry.total_quantity, Decimal::new(10, 0));

        assert_eq!(outcome_book.best_bid(), Some(Decimal::new(25, 2)));
        assert_eq!(outcome_book.best_ask(), None);
    }

    #[test]
    fn test_remove_order() {
        let price = Decimal::new(25, 3);
        let mut order_book = OutcomeBook::default();

        let order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id: get_random_uuid(),
            outcome: Outcome::YES,         // yes side,
            price,                         // 0.25
            quantity: Decimal::new(10, 0), // 10
            side: OrderSide::BUY,
            status: OrderStatus::UNSPECIFIED,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),

            order_type: OrderType::LIMIT,
        };
        order_book.add_order(&order);
        let price_level = order_book.bids.get(&price).unwrap();

        assert_eq!(order_book.bids.len(), 1);
        assert_eq!(price_level.total_quantity, Decimal::new(10, 0));

        order_book.remove_order(order.id, OrderSide::BUY, price);

        let price_level = order_book.bids.get(&price);

        assert_eq!(order_book.bids.len(), 0);
        assert!(price_level.is_none());
    }

    #[test]
    fn test_update_order() {
        let price = Decimal::new(25, 2); // 0.25
        let created_at = get_created_at();
        let id = get_random_uuid();
        let market_id = get_random_uuid();
        let updated_at = get_created_at();
        let user_id = get_random_uuid();
        let quantity = Decimal::new(10, 0);
        let order = Order {
            created_at,
            filled_quantity: Decimal::ZERO,
            id,
            market_id,
            outcome: Outcome::YES,         // yes side,
            price,                         // 0.25
            quantity: Decimal::new(10, 0), // 10
            side: OrderSide::BUY,
            status: OrderStatus::UNSPECIFIED,
            updated_at,
            user_id,
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();

        outcome_book.add_order(&order);

        assert_eq!(outcome_book.bids.len(), 1);

        let price_level = outcome_book.bids.get(&price).unwrap();
        assert_eq!(price_level.total_quantity, quantity);

        // updating order
        outcome_book._update_order_filled_quantity(id, OrderSide::BUY, price, Decimal::new(5, 0));

        let price_level = outcome_book.bids.get(&price).unwrap();
        assert_eq!(price_level.total_quantity, Decimal::new(5, 0));
        let price_order = price_level.orders.get(0).unwrap();
        assert_eq!(price_order.filled_quantity, Decimal::new(5, 0));
    }

    #[test]
    fn test_match_order() {
        let market_id = get_random_uuid();

        let buy_order_1 = Order {
            order_type: OrderType::LIMIT,

            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(25, 2),    // 0.25
            quantity: Decimal::new(10, 0), // 10
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
        }; // 0.25 buy - 10 qty (buy)
        let buy_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(20, 2),   // 0.25
            quantity: Decimal::new(3, 0), // 10
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,

            user_id: get_random_uuid(),
        }; // 0.20 buy - 3 qty (buy)
        let buy_order_3 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(23, 2),   // 0.25
            quantity: Decimal::new(4, 0), // 10
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,

            user_id: get_random_uuid(),
        }; // 0.23 buy - 4 qty (buy)

        let mut sell_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(20, 2),
            quantity: Decimal::new(15, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,

            user_id: get_random_uuid(),
        }; // 0.20 - 15 qty (sell)
        let mut outcome_book = OutcomeBook::default();

        outcome_book.add_order(&buy_order_1);
        outcome_book.add_order(&buy_order_2);
        outcome_book.add_order(&buy_order_3);

        let resp = outcome_book.match_order(&mut sell_order_1);
        // NEED TO PERFORM POST UPDATES ON ADDED ORDERS....
        let order_book_entry = outcome_book.bids.get(&dec!(0.20));
        assert!(order_book_entry.is_some());
        let order_book_entry = order_book_entry.unwrap();
        assert!(order_book_entry.orders.len() == 1);
        assert!(order_book_entry.orders[0].filled_quantity == dec!(1));

        assert_eq!(sell_order_1.status, OrderStatus::FILLED);
        assert_eq!(resp.len(), 3);

        // Verify matching happened in price-time priority order
        assert_eq!(resp[0].opposite_order_id, buy_order_1.id); // Best price (0.25) first
        assert_eq!(resp[1].opposite_order_id, buy_order_3.id); // Second best price (0.23)
        assert_eq!(resp[2].opposite_order_id, buy_order_2.id); // Third best price (0.20)
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
            price: Decimal::new(25, 2),   // 0.25
            quantity: Decimal::new(5, 0), // 5
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
            price: Decimal::new(20, 2),    // 0.20
            quantity: Decimal::new(10, 0), // 10
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::OPEN);
        assert_eq!(sell_order.filled_quantity, Decimal::new(5, 0));
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].matched_quantity, Decimal::new(5, 0)); // matched quantity
    }

    #[test]
    fn test_match_multiple_orders_same_price() {
        let market_id = get_random_uuid();
        let price = Decimal::new(25, 2); // 0.25

        // 3 buy orders at the same price
        let buy_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price,
            quantity: Decimal::new(5, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let buy_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price,
            quantity: Decimal::new(3, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let buy_order_3 = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price,
            quantity: Decimal::new(2, 0),
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
            price,
            quantity: Decimal::new(7, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order_1);
        outcome_book.add_order(&buy_order_2);
        outcome_book.add_order(&buy_order_3);

        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::FILLED);
        assert_eq!(resp.len(), 2); // Should match with the first two orders
        assert_eq!(resp[0].opposite_order_id, buy_order_1.id);
        assert_eq!(resp[1].opposite_order_id, buy_order_2.id);
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
            price: Decimal::new(20, 2), // 0.20
            quantity: Decimal::new(10, 0),
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
            price: Decimal::new(25, 2), // 0.25 (higher than buy price)
            quantity: Decimal::new(10, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::OPEN);
        assert_eq!(sell_order.filled_quantity, Decimal::ZERO);
        assert_eq!(resp.len(), 0);
    }

    #[test]
    fn test_large_order_book() {
        let market_id = get_random_uuid();
        let user_id = get_random_uuid();

        let mut outcome_book = OutcomeBook::default();

        // Add 1000 buy orders at different prices
        for i in 1..=1000 {
            let buy_order = Order {
                created_at: get_created_at(),
                filled_quantity: Decimal::ZERO,
                id: get_random_uuid(),
                market_id,
                outcome: Outcome::YES,
                price: Decimal::new(i, 4), // range is 0.0001 to 1.0000
                quantity: Decimal::new(1, 0),
                side: OrderSide::BUY,
                status: OrderStatus::OPEN,
                updated_at: get_created_at(),
                order_type: OrderType::LIMIT,

                user_id,
            };
            outcome_book.add_order(&buy_order);
        }

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(5, 2),      // 0.05
            quantity: Decimal::new(500, 0), // Match with 500 highest bids
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::FILLED);
        assert_eq!(resp.len(), 500);
        assert_eq!(sell_order.filled_quantity, Decimal::new(500, 0));
    }

    #[test]
    fn test_already_partially_filled_order() {
        let market_id = get_random_uuid();

        let buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(25, 2), // 0.25
            quantity: Decimal::new(10, 0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::new(5, 0), // Already filled 5 units
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(20, 2), // 0.20
            quantity: Decimal::new(10, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::FILLED);
        assert_eq!(sell_order.filled_quantity, Decimal::new(10, 0));
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].matched_quantity, Decimal::new(5, 0)); // Only needed to match 5 more
    }

    #[test]
    fn test_empty_order_book() {
        let market_id = get_random_uuid();

        let mut sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: Decimal::ZERO,
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: Decimal::new(20, 2),
            quantity: Decimal::new(10, 0),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let mut outcome_book = OutcomeBook::default();
        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::OPEN);
        assert_eq!(sell_order.filled_quantity, Decimal::ZERO);
        assert_eq!(resp.len(), 0);
    }

    #[test]
    fn test_db_matching_order_issue() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        let buy_order_one = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.61),
            quantity: dec!(3),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };
        let buy_order_one_1 = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.61),
            quantity: dec!(3),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };
        let buy_order_one_2 = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.61),
            quantity: dec!(3),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&buy_order_one);
        outcome_book.add_order(&buy_order_one_1);
        outcome_book.add_order(&buy_order_one_2);

        let mut matching_sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.61),
            quantity: dec!(3),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };
        let matches = outcome_book.match_order(&mut matching_sell_order);
        assert_eq!(matches.len(), 1);
        let price_level = outcome_book.bids.get(&dec!(0.61)).unwrap();
        assert_eq!(price_level.orders.len(), 2); // matched 1 order so 3 - 1 = 2
    }

    #[test]
    fn test_market_order_with_budget() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        // Add multiple sell orders at different prices
        let sell_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.50),  // $0.50 per share
            quantity: dec!(10), // 10 shares available
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let sell_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.60),  // $0.60 per share
            quantity: dec!(15), // 15 shares available
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let sell_order_3 = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.80),  // $0.80 per share
            quantity: dec!(20), // 20 shares available
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order_1);
        outcome_book.add_order(&sell_order_2);
        outcome_book.add_order(&sell_order_3);

        // Create a market buy order with $10 budget
        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0),    // Market orders have zero price initially
            quantity: dec!(0), // Will be calculated based on budget
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let budget = dec!(10); // $10 budget
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // With $10 budget:
        // - First 10 shares at $0.50 = $5.00 (consume all of sell_order_1)
        // - Next 8.33 shares at $0.60 = $5.00 (partial consume of sell_order_2)
        // Total: 18.33 shares for $10

        assert_eq!(market_buy_order.price, dec!(0)); // Market order should have zero price
        assert_eq!(market_buy_order.quantity, dec!(10) + dec!(5) / dec!(0.60)); // Expected quantity based on budget

        assert_eq!(matches.len(), 2); // Should match with 2 orders
        assert_eq!(matches[0].price, dec!(0.50)); // First match at $0.50
        assert_eq!(matches[1].price, dec!(0.60)); // Second match at $0.60
    }

    #[test]
    fn test_market_order_partial_budget_consumption() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        // Add a single sell order
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25),   // $0.25 per share
            quantity: dec!(100), // 100 shares available
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

        // Create market order with $5 budget
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
            order_type: OrderType::LIMIT,
        };

        let budget = dec!(5); // $5 budget
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // With $5 budget at $0.25/share = 20 shares
        assert_eq!(market_buy_order.quantity, dec!(20));
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_quantity, dec!(20));
        assert_eq!(market_buy_order.status, OrderStatus::FILLED);
    }

    #[test]
    fn test_market_sell_order_with_budget() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        // Add multiple buy orders at different prices
        let buy_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.80), // $0.80 per share (highest bid)
            quantity: dec!(5),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let buy_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.70), // $0.70 per share
            quantity: dec!(10),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&buy_order_1);
        outcome_book.add_order(&buy_order_2);

        // Create market sell order
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
            order_type: OrderType::LIMIT,
        };

        let budget = dec!(11); // $11 target revenue
        let matches = outcome_book.create_market_order(&mut market_sell_order, budget);

        // With $11 target:
        // - Sell 5 shares at $0.80 = $4.00
        // - Sell 10 shares at $0.70 = $7.00
        // Total: 15 shares for $11.00

        assert_eq!(matches.len(), 2);
        assert_eq!(market_sell_order.status, OrderStatus::FILLED);
    }

    #[test]
    fn test_market_order_insufficient_liquidity() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        // Add small sell order
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(1.00),
            quantity: dec!(2), // Only 2 shares available
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

        // Try to create market order with large budget
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
            order_type: OrderType::LIMIT,
        };

        let budget = dec!(100); // $100 budget but only $2 worth available
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // Should only get 2 shares for $2, even with $100 budget
        assert_eq!(market_buy_order.quantity, dec!(2));
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_quantity, dec!(2));
    }

    #[test]
    fn test_market_order_zero_budget() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        // Add sell orders
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.50),
            quantity: dec!(10),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

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
            order_type: OrderType::LIMIT,
        };

        let budget = dec!(0); // Zero budget
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        assert_eq!(market_buy_order.quantity, dec!(0));
        assert_eq!(matches.len(), 0);
        assert_eq!(market_buy_order.status, OrderStatus::FILLED); // Zero quantity should be considered filled
    }

    #[test]
    fn test_market_order_negative_budget() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

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
            order_type: OrderType::LIMIT,
        };

        let budget = dec!(-10); // Negative budget
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        assert_eq!(market_buy_order.quantity, dec!(0));
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_market_order_with_pre_existing_price() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.50),
            quantity: dec!(10),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

        // Market order with non-zero price (should be reset to zero)
        let mut market_buy_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.75), // Non-zero price
            quantity: dec!(0),
            side: OrderSide::BUY,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let budget = dec!(5);
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        assert_eq!(market_buy_order.price, dec!(0)); // Should be reset to zero
        assert_eq!(market_buy_order.quantity, dec!(10)); // $5 / $0.50 = 10 shares
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_market_order_with_partially_filled_book_orders() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        // Add sell order with some already filled quantity
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(3), // Already partially filled
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.50),
            quantity: dec!(10), // Total quantity is 10, but 3 already filled, so 7 remaining
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

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
            order_type: OrderType::LIMIT,
        };

        let budget = dec!(5); // $5 budget
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // Should only get 7 remaining shares (not 10)
        assert_eq!(market_buy_order.quantity, dec!(7)); // Only 7 shares available
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_quantity, dec!(7));
    }

    #[test]
    fn test_market_order_exact_budget_match() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.25),  // $0.25 per share
            quantity: dec!(20), // 20 shares = $5 total
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

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
            order_type: OrderType::LIMIT,
        };

        let budget = dec!(5); // Exactly matches all available shares
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        assert_eq!(market_buy_order.quantity, dec!(20));
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_quantity, dec!(20));
        assert_eq!(market_buy_order.status, OrderStatus::FILLED);
    }

    #[test]
    fn test_market_order_very_small_budget() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.50),
            quantity: dec!(10),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

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
            order_type: OrderType::LIMIT,
        };

        let budget = dec!(0.01); // Very small budget
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // $0.01 / $0.50 = 0.02 shares
        assert_eq!(market_buy_order.quantity, dec!(0.02));
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_quantity, dec!(0.02));
    }

    #[test]
    fn test_market_order_multiple_levels_fractional_shares() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        // Multiple price levels with different quantities
        let sell_order_1 = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.30), // $0.30 per share
            quantity: dec!(5), // 5 shares = $1.50
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        let sell_order_2 = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.40), // $0.40 per share
            quantity: dec!(3), // 3 shares = $1.20
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order_1);
        outcome_book.add_order(&sell_order_2);

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
            order_type: OrderType::LIMIT,
        };

        let budget = dec!(2); // $2 budget
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // First 5 shares at $0.30 = $1.50, remaining $0.50 gets 1.25 shares at $0.40
        // Total: 5 + 1.25 = 6.25 shares
        assert_eq!(market_buy_order.quantity, dec!(5) + dec!(0.50) / dec!(0.40));
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_market_order_empty_book() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

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
            order_type: OrderType::LIMIT,
        };

        let budget = dec!(100); // Large budget but no orders in book
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        assert_eq!(market_buy_order.quantity, dec!(0));
        assert_eq!(matches.len(), 0);
        assert_eq!(market_buy_order.status, OrderStatus::FILLED); // Zero quantity is considered filled
    }

    #[test]
    fn test_market_order_self_matching_prevention() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();
        let user_id = get_random_uuid(); // Same user ID

        // Add sell order from same user
        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.50),
            quantity: dec!(10),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            order_type: OrderType::LIMIT,

            user_id, // Same user ID
        };

        outcome_book.add_order(&sell_order);

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
            order_type: OrderType::LIMIT,

            user_id, // Same user ID - should not match with own order
        };

        let budget = dec!(5);
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // Should not match with own order
        assert_eq!(market_buy_order.quantity, dec!(0));
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_market_order_precision_handling() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.333333), // Price with many decimals
            quantity: dec!(100),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),

            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

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
            order_type: OrderType::LIMIT,
        };

        let budget = dec!(10);
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // Should handle precision correctly
        assert!(market_buy_order.quantity > dec!(0));
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_market_order_status_progression() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.50),
            quantity: dec!(5),
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

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
            order_type: OrderType::LIMIT,
        };

        // Test partial fill
        let budget = dec!(1); // Only $1, can buy 2 shares at $0.50
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        assert_eq!(market_buy_order.quantity, dec!(2));
        assert_eq!(market_buy_order.status, OrderStatus::FILLED); // Market orders should be filled immediately
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_quantity, dec!(2));
    }

    #[test]
    fn test_market_order_large_numbers() {
        let mut outcome_book = OutcomeBook::default();
        let market_id = Uuid::new_v4();

        let sell_order = Order {
            created_at: get_created_at(),
            filled_quantity: dec!(0),
            id: get_random_uuid(),
            market_id,
            outcome: Outcome::YES,
            price: dec!(0.001),      // Very small price
            quantity: dec!(1000000), // Large quantity
            side: OrderSide::SELL,
            status: OrderStatus::OPEN,
            updated_at: get_created_at(),
            user_id: get_random_uuid(),
            order_type: OrderType::LIMIT,
        };

        outcome_book.add_order(&sell_order);

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
            order_type: OrderType::LIMIT,
        };

        let budget = dec!(100); // $100 budget
        let matches = outcome_book.create_market_order(&mut market_buy_order, budget);

        // $100 / $0.001 = 100,000 shares
        assert_eq!(market_buy_order.quantity, dec!(100000));
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].matched_quantity, dec!(100000));
    }
}
