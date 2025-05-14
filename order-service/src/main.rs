use async_nats::jetstream;
use futures_util::stream::StreamExt;
use state::AppState;
use utility_helpers::log_info;
pub mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = AppState::new().await?;
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
                    log_info!("Received message: {:?}", std::str::from_utf8(&msg.payload)?);

                    msg.ack()
                        .await
                        .map_err(|_| "Failed to acknowledge message".to_string())?;
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
