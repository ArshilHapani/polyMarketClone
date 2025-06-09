use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::{
    sync::{RwLock, broadcast, mpsc},
    time::interval,
};
use utility_helpers::{log_info, log_warn};
use uuid::Uuid;

use crate::ws_utils::{BroadcastMessage, ClientMessage, MessagePayload, SubscriptionChannels};

pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<SubscriptionChannels, ProcessHandle>>>,
    subscribers: Arc<
        RwLock<HashMap<SubscriptionChannels, HashMap<Uuid, broadcast::Sender<BroadcastMessage>>>>,
    >,
}

pub struct ProcessHandle {
    pub channel: SubscriptionChannels,
    pub task_handler: tokio::task::JoinHandle<()>,
    pub broadcaster: broadcast::Sender<BroadcastMessage>,
    pub control_tx: mpsc::Sender<ProcessControl>,
    pub subscriber_count: Arc<RwLock<usize>>,
}

#[derive(Debug)]
pub enum ProcessControl {
    Stop,
    UpdateConfig(serde_json::Value),
    AddSubscriber(Uuid),
    RemoveSubscriber(Uuid),
}
// PROCESS manager completed, store it in app state and use in connection handler
impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn handle_client_message(
        &self,
        client_id: Uuid,
        message: ClientMessage,
    ) -> Result<Option<String>, String> {
        match message.payload {
            MessagePayload::Subscribe { channel, params } => {
                self.subscribe_client(client_id, &channel, params).await
            }
            MessagePayload::Unsubscribe { channel } => {
                self.unsubscribe_client(client_id, &channel).await
            }
            _ => Ok(Some("Message proceed".to_string())),
        }
    }

    pub async fn subscribe_client(
        &self,
        client_id: Uuid,
        channel: &str,
        params: serde_json::Value,
    ) -> Result<Option<String>, String> {
        if let Some(channel_type) = SubscriptionChannels::from_str(channel) {
            self.ensure_process_exists(&channel_type, params).await?;

            let mut subscribers = self.subscribers.write().await;
            let channel_subscribers = subscribers
                .entry(channel_type.clone())
                .or_insert_with(HashMap::new);

            let (tx, _rx) = broadcast::channel(1000);
            channel_subscribers.insert(client_id, tx.clone());

            if let Some(process) = self.processes.read().await.get(&channel_type) {
                let _ = process
                    .control_tx
                    .send(ProcessControl::AddSubscriber(client_id))
                    .await;
                *process.subscriber_count.write().await += 1;
            }
        }

        log_info!("Subscribed client {} to channel {}", client_id, channel);
        Ok(Some(format!("Subscribed to channel: {}", channel)))
    }

    pub async fn unsubscribe_client(
        &self,
        client_id: Uuid,
        channel: &str,
    ) -> Result<Option<String>, String> {
        let channel_type = SubscriptionChannels::from_str(channel)
            .ok_or_else(|| format!("Invalid channel: {channel}"))?;

        let mut subscribers = self.subscribers.write().await;
        if let Some(channel_subscribers) = subscribers.get_mut(&channel_type) {
            channel_subscribers.remove(&client_id);

            if let Some(process) = self.processes.read().await.get(&channel_type) {
                let _ = process
                    .control_tx
                    .send(ProcessControl::RemoveSubscriber(client_id))
                    .await;

                let mut count = process.subscriber_count.write().await;
                if *count > 0 {
                    *count -= 1;
                }

                if *count == 0 {
                    log_info!("No subscriber left for the channel: {}", channel_type);
                }
            }
        }

        log_info!("Unsubscribed client {} from channel {}", client_id, channel);
        Ok(Some(format!("Unsubscribed from channel: {}", channel)))
    }

    async fn ensure_process_exists(
        &self,
        channel: &SubscriptionChannels,
        params: serde_json::Value,
    ) -> Result<(), String> {
        let mut processes = self.processes.write().await;

        if !processes.contains_key(channel) {
            log_info!("Starting new process for channel: {channel}");

            let (broadcaster, _) = broadcast::channel(1000);
            let (control_tx, control_rx) = mpsc::channel(100);
            let subscriber_count = Arc::new(RwLock::new(0));

            let task_handler = self
                .start_channel_process(channel.clone(), broadcaster.clone(), control_rx, params)
                .await;

            let process_handler = ProcessHandle {
                channel: channel.clone(),
                task_handler,
                broadcaster,
                control_tx,
                subscriber_count,
            };

            processes.insert(channel.clone(), process_handler);
        }

        Ok(())
    }

    async fn start_channel_process(
        &self,
        channel_type: SubscriptionChannels,
        broadcaster: broadcast::Sender<BroadcastMessage>,
        control_rx: mpsc::Receiver<ProcessControl>,
        params: serde_json::Value,
    ) -> tokio::task::JoinHandle<()> {
        // let subscribers = self.subscribers.clone();
        tokio::spawn(async move {
            match channel_type {
                SubscriptionChannels::PriceUpdates(market_id) => {
                    Self::price_update_process(market_id, broadcaster, control_rx, params).await;
                }
            }

            log_info!("Process for channel {} has stopped", channel_type);
        })
    }

    //////////// Channel handlers //////////////
    async fn price_update_process(
        market_id: Uuid,
        broadcaster: broadcast::Sender<BroadcastMessage>,
        mut control_rx: mpsc::Receiver<ProcessControl>,
        _params: serde_json::Value,
    ) {
        let mut interval = interval(Duration::from_secs(1));
        let mut price = (0.4, 0.6);

        loop {
            tokio::select! {
                _ =  interval.tick() => {
                    price.0 += 0.01;
                    price.1 -= 0.01;

                    let message = BroadcastMessage {
                        channel: format!("price_updates:{market_id}"),
                        data: serde_json::json!({
                            "market_id": market_id,
                            "price":{
                                "yes_price": price.0,
                                "no_price": price.1
                            }
                        }),
                        timestamp: chrono::Utc::now(),
                    };

                    if broadcaster.send(message).is_err() {
                        log_warn!("No subscribers for channel: price_updates:{market_id}");
                    }
                }

                  control_msg = control_rx.recv() => {
                    match control_msg {
                        Some(ProcessControl::Stop) => {
                            log_info!("Stopping price update process for {}", market_id);
                            break;
                        }
                        Some(ProcessControl::UpdateConfig(config)) => {
                            log_info!("Updating config for price process: {:?}", config);
                            // Handle config updates
                        }
                        Some(ProcessControl::AddSubscriber(client_id)) => {
                            log_info!("New subscriber {} for price updates: {}", client_id, market_id);
                        }
                        Some(ProcessControl::RemoveSubscriber(client_id)) => {
                            log_info!("Subscriber {} left price updates: {}", client_id, market_id);
                        }
                        None => break,
                    }
                }
            };
        }
    }

    //////////// Cleanups ////////////////
    pub async fn cleanup_client(&self, client_id: Uuid) {
        log_info!("Cleaning up client: {client_id}");

        let mut subscribers = self.subscribers.write().await;
        let mut channels_to_check = Vec::new();

        for (channel_type, channel_subscribers) in subscribers.iter_mut() {
            if channel_subscribers.remove(&client_id).is_some() {
                channels_to_check.push(channel_type.clone());
            }
        }

        for channel_type in channels_to_check {
            if let Some(process) = self.processes.read().await.get(&channel_type) {
                let _ = process
                    .control_tx
                    .send(ProcessControl::RemoveSubscriber(client_id))
                    .await;

                let mut count = process.subscriber_count.write().await;
                if *count > 0 {
                    *count -= 1;
                }
            }
        }
    }

    pub async fn stop_channel(&self, channel: &SubscriptionChannels) -> Result<(), String> {
        let mut processes = self.processes.write().await;

        if let Some(process) = processes.remove(channel) {
            let _ = process.control_tx.send(ProcessControl::Stop).await;
            process.task_handler.abort();
            log_info!("Stopped process for channel: {}", channel);
        }
        self.subscribers.write().await.remove(channel);
        Ok(())
    }

    pub async fn stop_all_processes(&self) {
        log_info!("Stopping all processes...");

        let mut processes = self.processes.write().await;

        for (channel, process) in processes.drain() {
            let _ = process.control_tx.send(ProcessControl::Stop).await;
            process.task_handler.abort();
            log_info!("Stopped process for channel: {}", channel);
        }

        self.subscribers.write().await.clear();
    }

    pub async fn get_active_channels(&self) -> Vec<String> {
        self.processes
            .read()
            .await
            .keys()
            .map(|ct| ct.to_string())
            .collect()
    }

    pub async fn get_subscriber_count(&self, channel: &SubscriptionChannels) -> usize {
        if let Some(process) = self.processes.read().await.get(channel) {
            *process.subscriber_count.read().await
        } else {
            0
        }
    }
}
