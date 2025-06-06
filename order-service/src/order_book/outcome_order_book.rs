use std::collections::BTreeMap;

use db_service::schema::{
    enums::{OrderSide, OrderStatus},
    orders::Order,
};
use rust_decimal::Decimal;
use uuid::Uuid;

use super::{OrderBookEntry, PriceLevel};

#[derive(Debug, Default)]
pub(crate) struct OutcomeOrderBook {
    pub(crate) bids: BTreeMap<Decimal, PriceLevel>, // buyers
    pub(crate) asks: BTreeMap<Decimal, PriceLevel>, // sellers
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
                let mut new_orders = Vec::new(); //  creating new vector every time... instead of internal vector mutation (Fixed in V2)
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
