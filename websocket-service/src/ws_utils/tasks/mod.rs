use std::sync::Arc;

use tokio::sync::mpsc;

use crate::ws_utils::{SafeSender, SubscriptionChannel, process_manager_v2::ProcessMessage};

mod order_book_update;
mod price_update;

pub fn get_task_by_channel(
    channel: &SubscriptionChannel,
) -> fn(
    mpsc::Sender<ProcessMessage>,
    serde_json::Value,
    Arc<Vec<SafeSender>>,
) -> tokio::task::JoinHandle<()> {
    match channel {
        SubscriptionChannel::PriceUpdates(_) => |sender, value, clients| {
            tokio::spawn(
                async move { price_update::price_update_task(sender, value, clients).await },
            )
        },
        SubscriptionChannel::OrderBookUpdate(_) => |sender, value, _| {
            tokio::spawn(
                async move { order_book_update::order_book_update_task(sender, value).await },
            )
        },
    }
}
