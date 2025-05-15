use std::{str::FromStr, vec};

use crate::state::AppState;
use db_service::schema::{enums::OrderStatus, orders::Order};
use utility_helpers::log_info;
use uuid::Uuid;

pub async fn handle_orders(
    app_state: &AppState,
    order_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let order_id = Uuid::from_str(&order_id).map_err(|_| "Invalid order ID format".to_string())?;
    let mut order = Order::find_order_by_id_with_market(order_id, &app_state.db_pool)
        .await
        .map_err(|_| "Failed to find order".to_string())?;

    if order.status == OrderStatus::OPEN {
        log_info!("Order already open - {:?}", order.id);
        return Ok(());
    }

    let matches = {
        if let Some(liquidity_b) = order.liquidity_b {
            let mut order_book = app_state.order_book.write();
            // TODO - fix this shitty error
            order_book.process_order(order.into(), liquidity_b)
        } else {
            vec![]
        }
    };

    Ok(())
}
