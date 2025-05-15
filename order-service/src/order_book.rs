use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDateTime;
use db_service::schema::{
    enums::{OrderSide, OrderStatus, Outcome},
    orders::Order,
};
use rust_decimal::Decimal;
use uuid::Uuid;

pub(crate) struct GlobalOrderBook {
    pub(crate) markets: HashMap<Uuid, MarketOrderBook>,
}

#[derive(Debug, Default)]
pub(crate) struct MarketOrderBook {
    pub(crate) yes_book: OutcomeOrderBook,
    pub(crate) no_book: OutcomeOrderBook,
    pub(crate) market_id: Uuid,
    pub(crate) current_yes_price: Decimal,
    pub(crate) current_no_price: Decimal,
    pub(crate) liquidity_b: Decimal,
}

#[derive(Debug, Default)]
pub(crate) struct OutcomeOrderBook {
    pub(crate) bids: BTreeMap<Decimal, PriceLevel>,
    pub(crate) asks: BTreeMap<Decimal, PriceLevel>,
}

#[derive(Debug, Default)]
pub(crate) struct PriceLevel {
    pub(crate) orders: Vec<OrderBookEntry>,
    pub(crate) total_quantity: Decimal,
}

#[derive(Debug, Clone)]
struct OrderBookEntry {
    pub(crate) order_id: Uuid,
    pub(crate) user_id: Uuid,
    pub(crate) quantity: Decimal,
    pub(crate) filled_quantity: Decimal,
    pub(crate) timestamp: NaiveDateTime,
}

impl OutcomeOrderBook {
    // add an order to the book
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
        price_level.total_quantity += order.quantity;
    }

    // get best bid price
    pub(crate) fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().cloned()
    }

    // get best ask price
    pub(crate) fn best_ask(&self) -> Option<Decimal> {
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
                price_level.total_quantity -= removed_order.quantity;

                if price_level.orders.is_empty() {
                    price_map.remove(&price);
                }

                return true;
            }
        }
        false
    }

    // update order
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
                let quantity_change = new_filled_quantity - order.filled_quantity;

                order.filled_quantity = new_filled_quantity;
                price_level.total_quantity -= quantity_change;

                if price_level.total_quantity <= Decimal::ZERO {
                    price_map.remove(&price);
                }

                return true;
            }
        }
        false
    }

    // match order
    pub(crate) fn match_order(&mut self, order: &mut Order) -> Vec<(Uuid, Uuid, Decimal, Decimal)> {
        let mut matches: Vec<(Uuid, Uuid, Decimal, Decimal)> = Vec::new();

        let (opposite_side_book, is_buy) = match order.side {
            OrderSide::BUY => (&mut self.asks, true),
            OrderSide::SELL => (&mut self.bids, false),
        };

        let price_levels: Vec<(&Decimal, &mut PriceLevel)> = if is_buy {
            opposite_side_book.iter_mut().collect()
        } else {
            opposite_side_book.iter_mut().rev().collect()
        };

        let mut remaining_quantity = order.quantity - order.filled_quantity;
        let mut to_remove_price: Option<Decimal> = None;
        for (price, price_level) in price_levels {
            if (is_buy && order.price < *price) || (!is_buy && order.price > *price) {
                continue;
            }

            let mut i = 0;
            while i < price_level.orders.len() && remaining_quantity > Decimal::ZERO {
                let opposite_order = &mut price_level.orders[i];
                let opposite_remaining = opposite_order.quantity - opposite_order.filled_quantity;

                if opposite_remaining <= Decimal::ZERO {
                    i += 1;
                    continue;
                }

                let match_quantity = opposite_remaining.min(remaining_quantity);

                opposite_order.filled_quantity += match_quantity;
                remaining_quantity -= match_quantity;
                order.filled_quantity += match_quantity;

                matches.push((order.id, opposite_order.order_id, match_quantity, *price));

                if opposite_remaining == match_quantity {
                    i += 1;
                }

                if remaining_quantity == Decimal::ZERO {
                    break;
                }
            }

            price_level.total_quantity = price_level
                .orders
                .iter()
                .map(|o| o.quantity - o.filled_quantity)
                .sum();

            if price_level.total_quantity <= Decimal::ZERO {
                to_remove_price = Some(*price);
            }

            if remaining_quantity == Decimal::ZERO {
                break;
            }
        }

        if let Some(price) = to_remove_price {
            opposite_side_book.remove(&price);
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
            if let Some(best_bid) = self.yes_book.best_bid() {
                if let Some(best_ask) = self.yes_book.best_ask() {
                    self.current_yes_price = (best_bid + best_ask) / Decimal::new(2, 0);
                }
            }

            if let Some(best_bid) = self.no_book.best_bid() {
                if let Some(best_ask) = self.no_book.best_ask() {
                    self.current_no_price = (best_bid + best_ask) / Decimal::new(2, 0);
                }
            }
        }
    }

    pub(crate) fn calculate_total_funds(&self, outcome: Outcome) -> Decimal {
        match outcome {
            Outcome::YES => self
                .yes_book
                .bids
                .iter()
                .map(|(price, level)| *price * level.total_quantity)
                .sum(),
            Outcome::NO => self
                .no_book
                .bids
                .iter()
                .map(|(price, level)| *price * level.total_quantity)
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
