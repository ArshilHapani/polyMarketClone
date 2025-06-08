use std::{error::Error, str::FromStr, sync::Arc};

use db_service::schema::{
    enums::{OrderSide, OrderStatus, Outcome},
    orders::Order,
    user_holdings::UserHoldings,
    user_trades::UserTrades,
};
use rust_decimal::Decimal;
use utility_helpers::log_info;
use uuid::Uuid;

use crate::state::AppState;

pub async fn order_book_v2_handler(
    app_state: Arc<AppState>,
    order_id: String,
) -> Result<(), Box<dyn Error>> {
    let order_id = Uuid::from_str(&order_id).map_err(|_| "Invalid order ID format".to_string())?;
    let order = Order::find_order_by_id_with_market(order_id, &app_state.db_pool)
        .await
        .map_err(|e| format!("Failed to find order {:#?}", e))?;

    // open orders are already added to order book during initialization
    if order.status == OrderStatus::OPEN {
        log_info!("Order already open");
        return Ok(());
    }

    // working on unspecified status order
    let matched_order = {
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

        // updating current order filled quantity and status
        order_raw
            .update(&app_state.db_pool)
            .await
            .map_err(|e| format!("Failed to update order: {:#?}", e))?;

        matches
    };

    for match_item in matched_order {
        // update the opposite order's filled quantity
        let buy_order_id = match_item.order_id;
        let sell_order_id = match_item.opposite_order_id;
        let quantity = match_item.matched_quantity;
        let opposite_order_new_status = if match_item.opposite_order_filled_quantity
            == match_item.opposite_order_total_quantity
        {
            OrderStatus::FILLED
        } else {
            OrderStatus::OPEN
        };

        Order::update_order_status_and_filled_quantity(
            &app_state.db_pool,
            sell_order_id,
            opposite_order_new_status,
            match_item.opposite_order_filled_quantity,
        )
        .await
        .map_err(|e| format!("Failed to update opposite order: {:#?}", e))?;

        let (buyer_id, seller_id) = match order.side {
            OrderSide::BUY => {
                Order::get_buyer_and_seller_user_id(&app_state.db_pool, buy_order_id, sell_order_id)
                    .await
                    .map_err(|e| format!("Failed to get buyer and seller id: {:#?}", e))?
            }
            OrderSide::SELL => {
                Order::get_buyer_and_seller_user_id(&app_state.db_pool, sell_order_id, buy_order_id)
                    .await
                    .map_err(|e| {
                        format!("Failed to get buyer and seller id for SELL side: {:#?}", e)
                    })?
            }
        };

        let create_buyer_trade_future = UserTrades::create_user_trade(
            &app_state.db_pool,
            buy_order_id,
            sell_order_id,
            order.user_id,
            order.market_id,
            order.outcome,
            match_item.price,
            quantity,
        );
        let create_seller_trade_future = UserTrades::create_user_trade(
            &app_state.db_pool,
            sell_order_id,
            buy_order_id,
            buyer_id,
            order.market_id,
            order.outcome,
            match_item.price,
            quantity,
        );

        let user_a_quantity = if order.side == OrderSide::BUY {
            quantity
        } else {
            -quantity
        };
        let user_b_quantity = if order.side == OrderSide::SELL {
            quantity
        } else {
            -quantity
        };

        // there is a bug while updating holdings... fix this shit
        println!(
            "User A Quantity: {}, User B Quantity: {}",
            user_a_quantity, user_b_quantity
        );

        let update_user_holding_future = UserHoldings::update_user_holdings(
            &app_state.db_pool,
            order.user_id,
            order.market_id,
            user_a_quantity,
        );

        let seller_update_holding_future = UserHoldings::update_user_holdings(
            &app_state.db_pool,
            seller_id,
            order.market_id,
            user_b_quantity,
        );

        let (
            create_buyer_trade_result,
            update_user_holding_result,
            seller_update_holding_result,
            create_seller_trade_result,
        ) = tokio::join!(
            create_buyer_trade_future,
            update_user_holding_future,
            seller_update_holding_future,
            create_seller_trade_future
        );

        create_buyer_trade_result.map_err(|e| format!("Failed to create buyer trade {:#?}", e))?;
        update_user_holding_result
            .map_err(|e| format!("Failed to update user holdings {:#?}", e))?;
        seller_update_holding_result
            .map_err(|e| format!("Failed to update seller user holdings {:#?}", e))?;
        create_seller_trade_result
            .map_err(|e| format!("Failed to create seller trade {:#?}", e))?;
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

    log_info!(
        "Order processed.. YES Price: {}, NO Price: {}",
        yes_price,
        no_price
    );

    Ok(())
}
