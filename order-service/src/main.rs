use async_nats::jetstream;
use db_service::schema::{
    enums::{OrderStatus, Outcome},
    market::Market,
    orders::Order,
};
use futures_util::stream::StreamExt;
use order_book_handler::handle_orders;
use state::AppState;
use tokio::time;
use utility_helpers::{log_error, log_info};

mod order_book;
mod order_book_handler;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = initialize_app().await?;
    tracing_subscriber::fmt::init();

    log_info!("Connected to NATS JetStream");
    let stream = app_state.jetstream.get_stream("ORDERS").await?;
    let consumer = stream
        .get_or_create_consumer(
            "ORDERS",
            jetstream::consumer::pull::Config {
                durable_name: Some("order-worker".into()),
                filter_subject: "orders.created".into(),
                ..Default::default()
            },
        )
        .await?;

    log_info!("Listening for messages...");

    initialize_app().await?;

    loop {
        let mut messages = consumer
            .batch()
            .max_messages(10)
            .expires(std::time::Duration::from_secs(5))
            .messages()
            .await?;

        while let Some(message_result) = messages.next().await {
            match message_result {
                Ok(msg) => {
                    let order_id = std::str::from_utf8(&msg.payload)?.to_string();

                    log_info!("Received message: {:?}", order_id);

                    handle_orders(&app_state, order_id).await?;

                    msg.ack()
                        .await
                        .map_err(|_| "Failed to acknowledge message".to_string())?;
                }
                Err(e) => {
                    log_error!("Error receiving message: {}", e);
                }
            }
        }

        time::sleep(time::Duration::from_millis(100)).await;
    }
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
            if let Some(liquidity_b) = db_order.liquidity_b {
                let market = global_book.get_or_create_market(db_order.market_id, liquidity_b);

                if let Some(book) = market.get_order_book(&db_order.outcome) {
                    log_info!("Order added to book - {:?}", db_order.id);
                    let order: Order = db_order.into();
                    book.add_order(&order);
                }
            }
        }
    }
    Ok(app_state)
}
