use async_nats::jetstream;
use db_service::schema::{
    enums::{OrderStatus, Outcome},
    market::Market,
    orders::Order,
};
use futures_util::stream::StreamExt;
use order_book_handler::handle_orders;
use state::AppState;
use std::sync::Arc;
use utility_helpers::log_info;

mod order_book;
mod order_book_handler;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = initialize_app().await?;
    let app_state = Arc::new(app_state);
    tracing_subscriber::fmt::init();

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
        println!("Received order ID: {}", order_id);
        println!(
            "App state orders length: {:?}",
            app_state.order_book.read().markets
        );
        // handle_orders(app_state.clone(), order_id).await?;

        message
            .ack()
            .await
            .map_err(|_| "Failed to acknowledge message".to_string())?;
    }

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

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_handle_orders() {
        let app_state = initialize_app().await.unwrap();
        let app_state = Arc::new(app_state);
        let order_id = "d7bed0dd-e8e0-46e3-bfcf-72631bd8e36b".to_string();

        let result = handle_orders(app_state, order_id).await;

        assert!(result.is_ok());

        assert!(true);
    }
}
