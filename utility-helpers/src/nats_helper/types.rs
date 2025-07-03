use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize",
    deserialize = "T: serde::de::DeserializeOwned"
))]
pub struct OrderBookUpdateData<T> {
    pub yes_book: T,
    pub no_book: T,
    pub market_id: Uuid,
    pub timestamp: String,
}
