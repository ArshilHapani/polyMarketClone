use rust_decimal::Decimal;
use std::{str::FromStr, sync::Arc};
use uuid::Uuid;

use crate::state::AppState;
use db_service::schema::{
    enums::{OrderSide, OrderStatus, Outcome},
    orders::Order,
    user_holdings::UserHoldings,
    user_trades::UserTrades,
};
use utility_helpers::log_info;

pub async fn handle_orders(
    app_state: Arc<AppState>,
    order_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let order_id = Uuid::from_str(&order_id).map_err(|_| "Invalid order ID format".to_string())?;
    let order = Order::find_order_by_id_with_market(order_id, &app_state.db_pool)
        .await
        .map_err(|_| "Failed to find order".to_string())?;

    // default is UNSPECIFIED
    if order.status == OrderStatus::OPEN {
        log_info!("Order already open - {:?}", order.id);
        return Ok(());
    }

    let matched_orders = {
        let mut order_book = app_state.order_book.write();
        let mut order_raw = Order {
            id: order.id,
            status: OrderStatus::OPEN,
            created_at: order.created_at,
            filled_quantity: order.filled_quantity,
            market_id: order.market_id,
            outcome: order.outcome,
            price: order.price,
            quantity: order.quantity,
            side: order.side,
            updated_at: order.updated_at,
            user_id: order.user_id,
        };
        let matches = order_book.process_order(&mut order_raw, order.liquidity_b);
        order_raw
            .update(&app_state.db_pool)
            .await
            .map_err(|_| "Failed to update order".to_string())?;
        matches
    };

    // heavy db operations... (optimization needed)
    for (buy_order_id, sell_order_id, quantity, price) in matched_orders {
        let (buyer_id, seller_id) = if order.side == OrderSide::BUY {
            Order::get_buyer_and_seller_user_id(&app_state.db_pool, buy_order_id, sell_order_id)
                .await
                .map_err(|_| "Failed to get buyer and seller user ID".to_string())?
        } else {
            Order::get_buyer_and_seller_user_id(&app_state.db_pool, sell_order_id, buy_order_id)
                .await
                .map_err(|_| "Failed to get buyer and seller user ID".to_string())?
        };

        UserTrades::create_user_trade(
            &app_state.db_pool,
            buy_order_id,
            sell_order_id,
            order.user_id,
            order.market_id,
            order.outcome,
            price,
            quantity,
        )
        .await
        .map_err(|_| "Failed to create user trade".to_string())?;

        UserHoldings::update_user_holdings(
            &app_state.db_pool,
            order.user_id,
            order.market_id,
            order.outcome,
            quantity,
        )
        .await
        .map_err(|_| "Failed to update user holdings".to_string())?;

        UserHoldings::update_user_holdings(
            &app_state.db_pool,
            buyer_id,
            order.market_id,
            order.outcome,
            quantity,
        )
        .await
        .map_err(|_| "Failed to update buyer user holdings".to_string())?;

        UserHoldings::update_user_holdings(
            &app_state.db_pool,
            seller_id,
            order.market_id,
            order.outcome,
            -quantity,
        )
        .await
        .map_err(|_| "Failed to update seller user holdings".to_string())?;
    }

    let (yes_price, no_price) = {
        let order_book = app_state.order_book.read();
        let yes_price = order_book
            .get_market_price(&order.market_id, Outcome::YES)
            .unwrap_or_else(|| Decimal::new(5, 1));
        let no_price = order_book
            .get_market_price(&order.market_id, Outcome::NO)
            .unwrap_or_else(|| Decimal::new(5, 1));

        (yes_price, no_price)
    };

    // store yes_price and no_price in some time series db
    log_info!("yes_price: {:?}, no_price: {:?}", yes_price, no_price);

    Ok(())
}
