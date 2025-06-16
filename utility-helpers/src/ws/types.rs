use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    PriceUpdate,
    PricePoster,
}

impl ChannelType {
    pub fn from_str_serde(s: &str) -> Result<Self, serde_json::Error> {
        let json_str = format!("\"{s}\"");

        let deserialized_channel = serde_json::from_str::<ChannelType>(&json_str);
        return deserialized_channel;
    }

    pub fn to_str(&self) -> String {
        match self {
            ChannelType::PriceUpdate => "price_update".to_string(),
            ChannelType::PricePoster => "price_poster".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessagePayload {
    Subscribe {
        channel: String,
        params: serde_json::Value,
    },
    Unsubscribe {
        channel: String,
    },
    Post {
        channel: String,
        data: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    pub id: Option<String>, //TODO we can verify the client id with this id (TODO for now)
    pub payload: MessagePayload,
}
