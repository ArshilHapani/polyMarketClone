use std::{collections::HashMap, sync::Arc};

use serde::Serialize;
use tokio::sync::mpsc;
use utility_helpers::{log_error, log_info};
use uuid::Uuid;

use crate::ws_utils::{
    SafeSender, SubscriptionChannel, connection_handler::send_message, tasks::get_task_by_channel,
};

#[derive(Debug)]
pub struct ProcessManagerV2 {
    pub processes: HashMap<SubscriptionChannel, ProcessHandleV2>,
    pub subscribers: HashMap<SubscriptionChannel, HashMap<Uuid, SafeSender>>,
}

#[derive(Debug)]
pub struct ProcessHandleV2 {
    pub handler: tokio::task::JoinHandle<()>,
    pub tx: mpsc::Sender<ProcessMessage>,
    pub rx: mpsc::Receiver<ProcessMessage>,
}

#[derive(Debug)]
pub struct ProcessMessage {
    pub channel: SubscriptionChannel,
    pub data: serde_json::Value,
}

#[derive(Serialize, Debug)]
pub struct MessageOut {
    pub channel: String,
    pub data: serde_json::Value,
}

impl ProcessManagerV2 {
    pub fn new() -> Self {
        ProcessManagerV2 {
            processes: HashMap::new(),
            subscribers: HashMap::new(),
        }
    }

    pub fn create_process(&mut self, channel: SubscriptionChannel, payload: serde_json::Value) {
        if let Some(_) = self.processes.get(&channel) {
            log_info!(
                "Process for channel {:?} already exists, not creating a new one",
                channel
            );
            return;
        }

        let (process_tx, process_rx) = mpsc::channel(100);
        let task = get_task_by_channel(&channel);
        let safe_senders: Arc<Vec<SafeSender>> = Arc::new(
            self.subscribers
                .get(&channel)
                .map(|s| s.values().cloned().collect())
                .unwrap_or_default(),
        );
        let join_handler = task(process_tx.clone(), payload, safe_senders);
        let process_handler = ProcessHandleV2 {
            handler: join_handler,
            tx: process_tx,
            rx: process_rx,
        };
        self.processes.insert(channel.clone(), process_handler);
    }

    pub fn delete_process(&mut self, channel: &SubscriptionChannel) {
        if let Some(process) = self.processes.remove(&channel) {
            if !process.handler.is_finished() {
                process.handler.abort();
            }
            log_info!("Process for channel {:?} aborted", channel);
        }
    }

    pub fn listen_to_process_messages(&mut self) {
        // Collect channels to avoid borrowing self in the async block
        let channels: Vec<SubscriptionChannel> = self.processes.keys().cloned().collect();

        for channel in channels {
            // Remove the receiver from the process handle so we can move it into the task
            if let Some(process) = self.processes.get_mut(&channel) {
                // Take the receiver out of the struct
                let mut rx = std::mem::replace(&mut process.rx, mpsc::channel(1000).1);
                let channel_clone = channel.clone();
                // Clone subscribers HashMap for use in the async block
                let subscribers = self.subscribers.clone();

                tokio::spawn(async move {
                    log_info!("Listening to messages for channel: {:?}", channel_clone);

                    while let Some(message) = rx.recv().await {
                        log_info!(
                            "Received message on channel {:?}: {:?}",
                            channel_clone,
                            message
                        );

                        // subscribers
                        if let Some(subscribers) = subscribers.get(&channel_clone) {
                            for (_client_id, subscriber) in subscribers.iter() {
                                let message = serde_json::to_string(&MessageOut {
                                    channel: channel_clone.to_string(),
                                    data: message.data.clone(),
                                })
                                .unwrap_or_else(|e| {
                                    log_info!("Failed to serialize message: {:?}", e);
                                    return "{}".to_string();
                                });
                                if let Err(e) = send_message(subscriber, message.into()).await {
                                    log_error!("Failed to send message to subscriber: {:?}", e);
                                } else {
                                    log_info!(
                                        "Message sent to subscriber on channel: {:?}",
                                        channel_clone
                                    );
                                }
                            }
                        } else {
                            log_info!("No subscribers found for channel: {:?}", channel_clone);
                        }
                    }
                });
                log_info!(
                    "Started listening to process messages for channel: {:?}",
                    channel
                );
            }
        }
    }

    pub fn add_subscriber(
        &mut self,
        channel: SubscriptionChannel,
        client_id: Uuid,
        subscriber: SafeSender,
    ) {
        self.subscribers
            .entry(channel.clone())
            .or_default()
            .insert(client_id, subscriber);
        log_info!("Added subscriber to channel: {:?}", channel);
    }

    pub fn remove_subscriber_without_channel(&mut self, client_id: Uuid) {
        for (channel, subscribers) in self.subscribers.iter_mut() {
            if subscribers.remove(&client_id).is_some() {
                log_info!(
                    "Removed subscriber with ID {:?} from channel: {:?}",
                    client_id,
                    channel
                );
            }

            // If there are no subscribers left for this channel, remove the process
            if subscribers.is_empty() {
                let process_handler = self.processes.remove(channel);
                if let Some(handler) = process_handler {
                    if !handler.handler.is_finished() {
                        handler.handler.abort();
                        log_info!("Aborted process for channel: {:?}", channel);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use uuid::Uuid;

    use super::*;
    use crate::ws_utils::SubscriptionChannel;

    #[tokio::test]
    async fn test_process_manager_v2() {
        tracing_subscriber::fmt::init();

        let mut manager = ProcessManagerV2::new();
        let channel = SubscriptionChannel::PriceUpdates(Uuid::new_v4());
        let another_channel = SubscriptionChannel::OrderBookUpdate(Uuid::new_v4());
        let payload = serde_json::json!({"key": "value"});

        manager.create_process(channel.clone(), payload.clone());
        manager.create_process(another_channel.clone(), payload);
        assert!(manager.processes.contains_key(&channel));
        assert!(manager.processes.contains_key(&another_channel));
        {
            let process = manager.processes.get_mut(&channel).unwrap();
            assert!(!process.handler.is_finished());
            assert!(process.rx.try_recv().is_err()); // Ensure no messages are received yet

            // wait for 1 sec after tha message should be found in rx
            // tokio::time::sleep(tokio::time::Duration::from_secs(11)).await;
            // let message = process.rx.try_recv();
            // assert!(!message.is_err());
            // let mut buff = Vec::new();

            // process.rx.recv_many(&mut buff, 1000).await;

            // assert!(!buff.is_empty());
            // assert_eq!(buff.len(), 10);
            // println!("Received messages: {:?}", buff);
        }
        {
            let another_process = manager.processes.get_mut(&another_channel).unwrap();
            assert!(!another_process.handler.is_finished());
            assert!(another_process.rx.try_recv().is_err()); // Ensure no messages are received yet
        }

        manager.listen_to_process_messages();
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        manager.delete_process(&channel);
        manager.delete_process(&another_channel);
        assert!(!manager.processes.contains_key(&channel));
        assert!(!manager.processes.contains_key(&another_channel));
    }
}
