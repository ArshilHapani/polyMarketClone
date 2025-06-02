use async_nats::jetstream;
use db_service::schema::{enums::OrderStatus, market::Market, orders::Order};
use futures_util::stream::StreamExt;
use order_book_handler::handle_orders;
use state::AppState;
use std::sync::Arc;
use utility_helpers::log_info;

mod order_book;
mod order_book_handler;
mod order_book_v2;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let app_state = initialize_app().await?;
    let app_state = Arc::new(app_state);

    log_info!("Connected to NATS JetStream");

    let stream = app_state
        .jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: "ORDERS".to_string(),
            subjects: vec!["orders.>".to_string()],
            ..Default::default()
        })
        .await?;

    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("orders".to_string()),
            ..Default::default()
        })
        .await?;

    let mut messages = consumer.messages().await?.take(50);

    while let Some(message) = messages.next().await {
        let message = message?;
        let order_id = String::from_utf8(message.payload.to_vec())
            .map_err(|_| "Failed to convert payload to string".to_string())?;
        log_info!("Received order ID: {}", order_id);
        handle_orders(app_state.clone(), order_id).await?;

        message
            .ack()
            .await
            .map_err(|_| "Failed to acknowledge message".to_string())?;
    }

    Ok(())
}

async fn initialize_app() -> Result<AppState, Box<dyn std::error::Error>> {
    let app_state = AppState::new().await?;

    // get all open orders and push it into order book (this is because of restarting the service)
    let open_markets = Market::get_all_open_markets(&app_state.db_pool).await?;
    let open_orders = Order::get_all_open_orders(&app_state.db_pool).await?;
    {
        let mut global_book = app_state.order_book.write();

        for market in &open_markets {
            global_book.get_or_create_market(market.id, market.liquidity_b);
        }
        let mut loaded_markets = 0;
        for db_order in &open_orders {
            if db_order.status != OrderStatus::OPEN {
                continue;
            }
            let market = global_book.get_or_create_market(db_order.market_id, db_order.liquidity_b);

            if let Some(book) = market.get_order_book(&db_order.outcome) {
                let order: Order = db_order.into();
                loaded_markets += 1;
                book.add_order(&order);
            } else {
                println!("Else condition met for order {}", db_order.id);
            }
        }
        log_info!("Loaded {} open markets from db", loaded_markets);
    }
    Ok(app_state)
}
