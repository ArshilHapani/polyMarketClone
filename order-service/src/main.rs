use std::rc::Rc;

use db_service::schema::{
    enums::{OrderStatus, Outcome},
    market::Market,
    orders::Order,
};
use futures_util::stream::StreamExt;
use order_book_handler::handle_orders;
use state::AppState;
use utility_helpers::{log_error, log_info};

mod order_book;
mod order_book_handler;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = initialize_app().await?;
    let app_state = Rc::new(app_state);
    tracing_subscriber::fmt::init();

    log_info!("Connected to NATS JetStream");

    let subscription = app_state
        .nats_client
        .subscribe("orders.created")
        .await?
        .take(50);

    initialize_app().await?;
    log_info!("Application initialized");
    log_info!("Listening for order events...");

    subscription
        .for_each(|message| {
            let app_state = app_state.clone();
            async move {
                let order_id = String::from_utf8_lossy(&message.payload).to_string();
                println!("Received message: {:?}", order_id);
                let er = handle_orders(app_state, order_id).await;
                if let Err(e) = er {
                    log_error!("Error handling order: {:?}", e);
                }
            }
        })
        .await;

    Ok(())
}

async fn initialize_app() -> Result<AppState, Box<dyn std::error::Error>> {
    let app_state = AppState::new().await?;

    let open_markets = Market::get_all_open_markets(&app_state.db_pool).await?;
    let open_orders = Order::get_all_open_orders(&app_state.db_pool).await?;
    {
        let mut global_book = app_state.order_book.write();

        for market in &open_markets {
            global_book.get_or_create_market(market.id, market.liquidity_b);
        }

        for db_order in &open_orders {
            if db_order.status != OrderStatus::OPEN {
                continue;
            }
            if db_order.outcome == Outcome::UNSPECIFIED {
                continue;
            }
            let market = global_book.get_or_create_market(db_order.market_id, db_order.liquidity_b);

            if let Some(book) = market.get_order_book(&db_order.outcome) {
                log_info!("Order added to book - {:?}", db_order.id);
                let order: Order = db_order.into();
                book.add_order(&order);
            }
        }
    }
    Ok(app_state)
}
