use std::{error::Error, str::FromStr, sync::Arc};

use db_service::schema::{enums::OrderStatus, orders::Order};
use utility_helpers::{log_error, log_info};
use uuid::Uuid;

use crate::state::AppState;

pub async fn order_book_v2_handler(
    app_state: Arc<AppState>,
    order_id: String,
) -> Result<(), Box<dyn Error>> {
    let order_id = Uuid::from_str(&order_id).map_err(|_| "Invalid order ID format".to_string())?;
    let order = Order::find_order_by_id_with_market(order_id, &app_state.db_pool)
        .await
        .map_err(|e| {
            log_error!("Failed to find order: {:?}", e);
            "Failed to find order".to_string()
        })?;

    if order.status == OrderStatus::OPEN {
        log_info!("Order already open");
        return Ok(());
    }

    Ok(())
}
