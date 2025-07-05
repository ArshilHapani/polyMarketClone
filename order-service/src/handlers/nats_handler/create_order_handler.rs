use std::{error::Error, str::FromStr, sync::Arc};

use db_service::schema::{
    enums::{OrderSide, OrderStatus},
    orders::Order,
    user_holdings::UserHoldings,
    user_trades::UserTrades,
    users::User,
};
use utility_helpers::log_info;
use uuid::Uuid;

use crate::{handlers::update_services::update_service_state, state::AppState};

pub async fn create_order_handler(
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
    let (matched_order, updated_raw_order) = {
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

        let mut order_book = app_state.order_book.write();
        let matches = order_book.process_order(&mut order_raw, order.liquidity_b);

        // updating current order filled quantity and status
        (matches, order_raw)
    };
    updated_raw_order
        .update(&app_state.db_pool)
        .await
        .map_err(|e| format!("Failed to update order: {:#?}", e))?;

    for match_item in matched_order {
        // update the opposite order's filled quantity
        let current_order_id = match_item.order_id;
        let opposite_order_id = match_item.opposite_order_id;
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
            opposite_order_id,
            opposite_order_new_status,
            match_item.opposite_order_filled_quantity,
        )
        .await
        .map_err(|e| format!("Failed to update opposite order: {:#?}", e))?;

        let (current_order_user_id, opposite_order_user_id) = match order.side {
            OrderSide::BUY => Order::get_buyer_and_seller_user_id(
                &app_state.db_pool,
                current_order_id,
                opposite_order_id,
            )
            .await
            .map_err(|e| format!("Failed to get buyer and seller id: {:#?}", e))?,
            OrderSide::SELL => Order::get_buyer_and_seller_user_id(
                &app_state.db_pool,
                opposite_order_id,
                current_order_id,
            )
            .await
            .map_err(|e| format!("Failed to get buyer and seller id for SELL side: {:#?}", e))?,
        };

        /////// Database Transaction start ////////

        // here we are preferring to use db transaction instead of rust's parallel (tokio::join) operation processing (it compromises performance and perform sequential processing), +we can't share `tx` across async tasks parallelly
        let mut tx = app_state.db_pool.begin().await?;

        UserTrades::create_user_trade(
            &mut *tx,
            current_order_id,
            opposite_order_id,
            order.user_id,
            order.market_id,
            order.outcome,
            match_item.price,
            quantity,
        )
        .await?;

        UserTrades::create_user_trade(
            &mut *tx,
            current_order_id,
            opposite_order_id,
            current_order_user_id,
            order.market_id,
            order.outcome,
            match_item.price,
            quantity,
        )
        .await?;

        let (current_order_user_updated_holding, opposite_order_user_updated_holdings) =
            match order.side {
                OrderSide::BUY => (quantity, -quantity),
                OrderSide::SELL => (-quantity, quantity),
            };

        UserHoldings::update_user_holdings(
            &mut *tx,
            current_order_user_id,
            order.market_id,
            current_order_user_updated_holding,
        )
        .await?;

        UserHoldings::update_user_holdings(
            &mut *tx,
            opposite_order_user_id,
            order.market_id,
            opposite_order_user_updated_holdings,
        )
        .await?;

        // updating user balances
        User::update_two_users_balance(
            &mut *tx,
            current_order_user_id,
            opposite_order_user_id,
            match_item.matched_quantity * match_item.price,
            order.side,
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {:#?}", e))?;

        /////// Database Transaction end ////////
    }

    update_service_state(app_state.clone(), order.market_id).await
}

#[cfg(test)]
mod test {
    use std::{str::FromStr, time::Duration};

    use futures_util::SinkExt;
    use prost::Message;
    use proto_defs::proto_types::ws_common_types::{
        Channel, OperationType, Payload, WsData, WsMessage,
    };
    use rdkafka::{
        ClientConfig,
        producer::{FutureProducer, FutureRecord},
    };
    use rust_decimal_macros::dec;
    use serde_json::json;
    use tokio_tungstenite::{
        connect_async,
        tungstenite::{Message as WsMessageType, client::IntoClientRequest},
    };
    use utility_helpers::log_error;
    use uuid::Uuid;

    #[tokio::test]
    #[ignore = "just ignore"]
    async fn test_kafka_publishing() {
        let rd_kafka: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .set("message.timeout.ms", "10000")
            .create()
            .expect("Failed to create Kafka client");

        let record = FutureRecord::to("price-updates-test")
            .payload("test message 1")
            .key("test_key_1");

        println!("Record {record:?}");

        let res = rd_kafka.send(record, Duration::from_secs(0)).await;
        assert!(
            res.is_ok(),
            "Failed to send record to Kafka: {:?}",
            res.err()
        );
    }

    #[tokio::test]
    #[ignore = "just ignore"]
    async fn test_publish_data_to_clickhouse_client() {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:19092")
            .set("message.timeout.ms", "10000")
            .create()
            .expect("Failed to create Kafka client");

        let market_id = Uuid::new_v4().to_string();
        let ts = chrono::Utc::now().to_rfc3339();

        let msg = json!({
            "market_id": market_id,
            "yes_price": 0.4,
            "no_price": 0.6,
            "ts": ts,
        })
        .to_string();

        for _i in 0..10 {
            let record: FutureRecord<'_, String, String> =
                FutureRecord::to("price-updates").payload(&msg);
            let res = producer.send(record, Duration::from_secs(0)).await;
            assert!(
                res.is_ok(),
                "Failed to send record to Kafka: {:?}",
                res.err()
            );
        }
    }

    #[tokio::test]
    #[ignore = "just ignore"]
    async fn test_websocket_message() {
        let websocket_req = format!("ws://localhost:4010/ws")
            .into_client_request()
            .expect("Failed to create WebSocket request");
        let (mut stream, _) = connect_async(websocket_req)
            .await
            .expect("Failed to connect to WebSocket server");

        let real_market_id = Uuid::from_str("67df943a-09a5-4ddb-adeb-11042c37c324")
            .unwrap()
            .to_string();

        let market_data = serde_json::json!({
            "market_id": real_market_id,
            "yes_price": dec!(0.4).to_string(),
            "no_price": dec!(0.6).to_string(),
        })
        .to_string();

        let message = WsMessage {
            id: None,
            payload: Some(Payload {
                ops: OperationType::Post.into(),
                data: Some(WsData {
                    channel: Channel::Priceposter.into(),
                    params: market_data,
                }),
            }),
        };

        let bin_data = message.encode_to_vec();

        if let Err(e) = stream.send(WsMessageType::Binary(bin_data.into())).await {
            log_error!("Failed to send message to WebSocket: {:#?}", e);
        }
    }
}
