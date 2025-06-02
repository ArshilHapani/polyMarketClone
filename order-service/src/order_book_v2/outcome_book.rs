use std::collections::BTreeMap;

use chrono::NaiveDateTime;
use db_service::schema::{
    enums::{OrderSide, OrderStatus},
    orders::Order,
};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Default, Debug)]
pub(super) struct PriceLevel {
    orders: Vec<OrderBookEntry>, // should I consider using hashmap here for O(1) lookup
    pub(crate) total_quantity: Decimal,
}

#[derive(Debug)]
struct OrderBookEntry {
    user_id: Uuid,
    order_id: Uuid,
    price: Decimal,
    total_quantity: Decimal,
    filled_quantity: Decimal,
    timestamp: NaiveDateTime,
}

#[derive(Debug, Default)]
pub(super) struct OutcomeBook {
    pub(super) bids: BTreeMap<Decimal, PriceLevel>, // buyers side
    pub(super) asks: BTreeMap<Decimal, PriceLevel>, // sellers side
}

impl OutcomeBook {
    pub(super) fn add_order(&mut self, order: &Order) {
        let side = match order.side {
            OrderSide::BUY => &mut self.bids,
            OrderSide::SELL => &mut self.asks,
        };

        let price_level = side.entry(order.price).or_default();

        let entry = OrderBookEntry {
            filled_quantity: order.filled_quantity,
            order_id: order.id,
            price: order.price,
            timestamp: order.created_at,
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

    pub(super) fn update_order(
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

    pub(super) fn match_order(&mut self, order: &mut Order) -> Vec<(Uuid, Uuid, Decimal, Decimal)> {
        // order id, opposite order id, matched quantity, price
        let mut matches: Vec<(Uuid, Uuid, Decimal, Decimal)> = Vec::new();

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

        println!("Keys {:?}", keys);

        for price in keys {
            if (is_buy && price > order.price) || (!is_buy && price < order.price) {
                continue;
            }

            if let Some(price_level) = book.get_mut(&price) {
                let mut new_orders = Vec::new();

                for mut opposite_order in price_level.orders.drain(..) {
                    let opp_remaining =
                        opposite_order.total_quantity - opposite_order.filled_quantity;
                    if opp_remaining <= Decimal::ZERO {
                        continue;
                    }

                    let match_qty = remaining.min(opp_remaining);

                    ///// ATOMIC Operation START
                    opposite_order.filled_quantity += match_qty;
                    order.filled_quantity += match_qty;
                    remaining -= match_qty;

                    matches.push((order.id, opposite_order.order_id, match_qty, price));

                    if opposite_order.filled_quantity < opposite_order.total_quantity {
                        new_orders.push(opposite_order);
                    }
                    if remaining == Decimal::ZERO {
                        break;
                    }
                    ///// ATOMIC Operation END
                }

                price_level.orders = new_orders;
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
}

#[cfg(test)]
mod test {
    use db_service::schema::enums::Outcome;

    use super::*;

    fn get_created_at() -> NaiveDateTime {
        chrono::Utc::now().naive_local()
    }
    fn get_random_uuid() -> Uuid {
        Uuid::new_v4()
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
        assert_eq!(order_book_entry.price, price);
        assert_eq!(order_book_entry.filled_quantity, Decimal::ZERO);
        assert_eq!(order_book_entry.total_quantity, Decimal::new(10, 0));
        assert_eq!(order_book_entry.timestamp, created_at);

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
        };

        let mut outcome_book = OutcomeBook::default();

        outcome_book.add_order(&order);

        assert_eq!(outcome_book.bids.len(), 1);

        let price_level = outcome_book.bids.get(&price).unwrap();
        assert_eq!(price_level.total_quantity, quantity);

        // updating order
        outcome_book.update_order(id, OrderSide::BUY, price, Decimal::new(5, 0));

        let price_level = outcome_book.bids.get(&price).unwrap();
        assert_eq!(price_level.total_quantity, Decimal::new(5, 0));
        let price_order = price_level.orders.get(0).unwrap();
        assert_eq!(price_order.filled_quantity, Decimal::new(5, 0));
    }

    #[test]
    fn test_match_order() {
        let market_id = get_random_uuid();

        let buy_order_1 = Order {
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
            user_id: get_random_uuid(),
        }; // 0.20 - 15 qty (sell)
        let mut outcome_book = OutcomeBook::default();

        outcome_book.add_order(&buy_order_1);
        outcome_book.add_order(&buy_order_2);
        outcome_book.add_order(&buy_order_3);

        let resp = outcome_book.match_order(&mut sell_order_1);

        assert_eq!(sell_order_1.status, OrderStatus::FILLED);
        assert_eq!(resp.len(), 3);

        // Verify matching happened in price-time priority order
        assert_eq!(resp[0].1, buy_order_1.id); // Best price (0.25) first
        assert_eq!(resp[1].1, buy_order_3.id); // Second best price (0.23)
        assert_eq!(resp[2].1, buy_order_2.id); // Third best price (0.20)
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
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::OPEN);
        assert_eq!(sell_order.filled_quantity, Decimal::new(5, 0));
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].2, Decimal::new(5, 0)); // matched quantity
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
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order_1);
        outcome_book.add_order(&buy_order_2);
        outcome_book.add_order(&buy_order_3);

        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::FILLED);
        assert_eq!(resp.len(), 2); // Should match with the first two orders
        assert_eq!(resp[0].1, buy_order_1.id);
        assert_eq!(resp[1].1, buy_order_2.id);
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
                price: Decimal::new(i, 2),
                quantity: Decimal::new(1, 0),
                side: OrderSide::BUY,
                status: OrderStatus::OPEN,
                updated_at: get_created_at(),
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
        };

        let mut outcome_book = OutcomeBook::default();
        outcome_book.add_order(&buy_order);

        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::FILLED);
        assert_eq!(sell_order.filled_quantity, Decimal::new(10, 0));
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].2, Decimal::new(5, 0)); // Only needed to match 5 more
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
        };

        let mut outcome_book = OutcomeBook::default();
        let resp = outcome_book.match_order(&mut sell_order);

        assert_eq!(sell_order.status, OrderStatus::OPEN);
        assert_eq!(sell_order.filled_quantity, Decimal::ZERO);
        assert_eq!(resp.len(), 0);
    }
}
