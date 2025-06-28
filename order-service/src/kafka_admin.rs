use rdkafka::{
    admin::{AdminOptions, NewTopic, TopicReplication},
    types::RDKafkaErrorCode,
};
use utility_helpers::{log_error, log_info};

use crate::state::AppState;

pub async fn ensure_topic_exists(
    app_state: &AppState,
    topic: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    {
        let read_guard = app_state.topic_cache.read().await;
        if read_guard.contains(topic) {
            return Ok(());
        }
    }

    let mut write_guard = app_state.topic_cache.write().await;
    let admin = app_state.kafka_admin.write().await;

    if !write_guard.contains(topic) {
        let new_topic = NewTopic::new(topic, 1, TopicReplication::Fixed(1));

        let res = admin
            .create_topics(&[new_topic], &AdminOptions::new())
            .await?;

        for r in res {
            match r {
                Ok(t) => log_info!("✅ Created topic {}", t),
                Err((t, e)) => match e {
                    RDKafkaErrorCode::TopicAlreadyExists => {
                        log_info!("🔵 Topic {} already exists", t);
                    }
                    _ => {
                        log_error!("⚠️ Failed to create topic {}: {}", t, e);
                        return Err(Box::new(e));
                    }
                },
            }
        }
        write_guard.insert(topic.to_string());
    }
    Ok(())
}
