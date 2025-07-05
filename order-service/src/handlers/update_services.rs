/*
 * This file is ued to update the state of services
 *
 * - Pushes the price data into clickhouse via kafka
 * - Updates the order book into clickhouse via kafka
 * - Send the data to the websocket which serves the users
 */

use std::{str::FromStr, sync::Arc, time::Duration};

use db_service::schema::enums::Outcome;
use futures_util::SinkExt;
use prost::Message;
use proto_defs::proto_types::ws_common_types::{
    Channel, OperationType, Payload, WsData, WsMessage,
};
use rdkafka::producer::FutureRecord;
use rust_decimal::Decimal;
use tokio_tungstenite::tungstenite::Message as WsMessageType;
use utility_helpers::{
    kafka_topics::KafkaTopics,
    log_error, log_info,
    message_pack_helper::serialize_to_message_pack,
    nats_helper::{NatsSubjects, types::OrderBookUpdateData},
    types::OrderBookDataStruct,
};
use uuid::Uuid;

use crate::state::AppState;

pub async fn update_service_state(
    app_state: Arc<AppState>,
    market_id: Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
    // variable declarations....
    let current_time = chrono::Utc::now();
    let producer = app_state.producer.read().await;

    ///// Sync code block star /////

    // market id validation and current market state
    let (yes_price, no_price, yes_orders_data, no_orders_data, required_market_subs) = {
        let order_book = app_state.order_book.read();

        let yes_price = order_book
            .get_market_price(&market_id, Outcome::YES)
            .unwrap_or_else(|| Decimal::new(5, 1));
        let no_price = order_book
            .get_market_price(&market_id, Outcome::NO)
            .unwrap_or_else(|| Decimal::new(5, 1));

        let yes_orders = order_book.get_orders(&market_id, Outcome::YES);
        let no_orders = order_book.get_orders(&market_id, Outcome::NO);

        // processing yes orders
        let yes_orders_data = if let Some(yes_orders) = yes_orders {
            yes_orders.get_order_book()
        } else {
            OrderBookDataStruct::default()
        };
        // processing no orders
        let no_orders_data = if let Some(no_orders) = no_orders {
            no_orders.get_order_book()
        } else {
            OrderBookDataStruct::default()
        };
        let market_subs_guard = app_state.market_subs.read();
        let required_market_subs = market_subs_guard.contains(&market_id);

        (
            // passing states from sync codeblock to async code block....
            yes_price,
            no_price,
            yes_orders_data,
            no_orders_data,
            required_market_subs,
        )
    };

    log_info!(
        "Order processed.. YES Price: {}, NO Price: {}",
        yes_price,
        no_price
    );

    //// Sync code block end /////

    let combined_data = OrderBookUpdateData {
        yes_book: yes_orders_data.clone(),
        no_book: no_orders_data.clone(),
        market_id: market_id,
        timestamp: current_time.to_rfc3339(),
    };

    ///// kafka processing /////

    let ts = current_time.to_rfc3339();
    let data_to_publish_for_price_update = serde_json::json!({
        "market_id": market_id.to_string(),
        "yes_price":yes_price.to_string(),
        "no_price": no_price.to_string(),
        "ts": ts,
    })
    .to_string();

    let data_to_publish_for_order_book_update = serde_json::json!({
        "market_id": market_id.to_string(),
        "yes_asks": yes_orders_data.asks,
        "yes_bids": yes_orders_data.bids,
        "no_asks": no_orders_data.asks,
        "no_bids": no_orders_data.bids,
        "ts": ts,
    })
    .to_string();

    let price_update_topic = KafkaTopics::PriceUpdates.to_string();
    let market_order_book_update_topic = KafkaTopics::MarketOrderBookUpdate.to_string();

    let market_id_str = market_id.to_string();

    let record_price_update = FutureRecord::to(&price_update_topic)
        .payload(&data_to_publish_for_price_update)
        .key(&market_id_str);
    let record_order_book_update = FutureRecord::to(&market_order_book_update_topic)
        .payload(&data_to_publish_for_order_book_update)
        .key(&market_id_str);

    let send_producer_future_price = producer.send(record_price_update, Duration::from_secs(0));
    let send_producer_future_order_book =
        producer.send(record_order_book_update, Duration::from_secs(0));

    /////////////////////////////////////////////////////////////////////////////////////

    //// NATS processing ////
    if required_market_subs {
        let message_pack_encoded = serialize_to_message_pack(&combined_data)?;

        // pushing message to queue
        let js_guard = app_state.jetstream.clone();

        if let Err(e) = js_guard
            .publish(
                NatsSubjects::MarketBookUpdate(market_id).to_string(),
                message_pack_encoded.into(),
            )
            .await
        {
            log_error!("Failed to publish order book update to JetStream: {:#?}", e);
        }
    }

    // sending message to websocket ///////
    let mut ws_publisher = app_state.ws_tx.write().await;

    let yes_price = f64::from_str(&yes_price.to_string())
        .map_err(|_| "Failed to parse yes price to f64".to_string())?;
    let no_price = f64::from_str(&no_price.to_string())
        .map_err(|_| "Failed to parse no price to f64".to_string())?;

    let market_data = serde_json::json!({
        "market_id": market_id,
        "yes_price": yes_price,
        "no_price": no_price,
        "timestamp": current_time.timestamp_millis(),
    })
    .to_string();

    let message = WsMessage {
        id: None,
        payload: Some(Payload {
            ops: OperationType::Post as i32,
            data: Some(WsData {
                channel: Channel::Priceposter as i32,
                params: market_data,
            }),
        }),
    };

    let bin_data = message.encode_to_vec();

    let ws_broadcast_future = ws_publisher.send(WsMessageType::Binary(bin_data.into()));

    let (
        send_producer_future_resp_price,
        send_producer_future_resp_order_book,
        ws_broadcast_future_result,
    ) = tokio::join!(
        send_producer_future_price,
        send_producer_future_order_book,
        ws_broadcast_future,
    );

    send_producer_future_resp_price
        .map_err(|e| format!("Failed to send record to Kafka: {:#?}", e))?;
    send_producer_future_resp_order_book
        .map_err(|e| format!("Failed to send record to Kafka: {:#?}", e))?;

    if let Err(e) = ws_broadcast_future_result {
        log_info!("Failed to send message to WebSocket: {:#?}", e);
    }

    Ok(())
}
