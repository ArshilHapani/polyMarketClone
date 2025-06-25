use db_service::schema::{enums::OrderStatus, orders::Order};
use state::AppState;
use std::sync::Arc;
use utility_helpers::{log_error, log_info};

use crate::nats_handler::handle_nats_message;

mod nats_handler;
mod order_book;
mod order_book_handler;
mod state;
mod ws_handler;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app_state = initialize_app().await?;
    let nats_app_state = Arc::clone(&app_state);
    let ws_app_state = Arc::clone(&app_state);

    log_info!("Connected to NATS JetStream");

    let nats_join_handler = tokio::spawn(async move {
        if let Err(e) = handle_nats_message(nats_app_state).await {
            log_error!("Error in NATS handler: {}", e);
        }
    });
    let ws_join_handler = tokio::spawn(async move {
        if let Err(e) = ws_handler::ws_handler(ws_app_state).await {
            log_error!("Error in WebSocket handler: {}", e);
        }
    });

    // try_join! because if either of the tasks fails then we want to stop the executions
    tokio::try_join!(nats_join_handler, ws_join_handler)
        .map_err(|e| format!("Error in tokio join: {}", e))?;

    Ok(())
}

async fn initialize_app() -> Result<Arc<AppState>, Box<dyn std::error::Error>> {
    let app_state = Arc::new(AppState::new().await?);

    let open_orders = Order::get_all_open_or_unspecified_orders(&app_state.db_pool).await?;
    {
        let mut global_book = app_state.order_book.write();

        let mut order_ctn = 0;
        // iterate over open orders
        for db_order in open_orders {
            if db_order.status != OrderStatus::OPEN {
                continue;
            }
            let liquidity_b = db_order.liquidity_b.clone();
            let mut order: Order = db_order.into();
            global_book.process_order(&mut order, liquidity_b);
            order_ctn += 1;
        }
        log_info!("Loaded {} open orders into the global book", order_ctn);
    }
    Ok(app_state)
}
