use rust_decimal::Decimal;

pub mod kafka_topics;
pub mod macros;
pub mod message_pack_helper;
pub mod nats_helper;
pub mod redis;
pub mod symmetric;
pub mod types;
pub mod ws;

pub const SHOW_LOGS: bool = true;

pub fn to_f64(value: Decimal) -> Option<f64> {
    let value_str = value.to_string();
    let parsed_value = value_str.parse::<f64>();
    match parsed_value {
        Ok(v) => Some(v),
        Err(_) => None,
    }
}
