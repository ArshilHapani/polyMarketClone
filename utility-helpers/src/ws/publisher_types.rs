use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::ChannelType;

pub struct GenericWrapper<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub channel: ChannelType,
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PricePosterDataStruct {
    pub market_id: Uuid,
    pub yes_price: Decimal,
    pub no_price: Decimal,
}

// REUSABLE TYPES ACROSS SERVICES --- till here
