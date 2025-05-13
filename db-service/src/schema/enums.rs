use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "\"polymarket\".\"market_status\"")]
pub enum MarketStatus {
    OPEN,
    CLOSED,
    SETTLED,
}
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "\"polymarket\".\"outcome\"", rename_all = "lowercase")]
pub enum Outcome {
    YES,
    NO,
    INVALID,
    UNSPECIFIED,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "\"polymarket\".\"order_side\"", rename_all = "lowercase")]
pub enum OrderSide {
    BUY,
    SELL,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq)]
#[sqlx(
    type_name = "\"polymarket\".\"order_status\"",
    rename_all = "lowercase"
)]

pub enum OrderStatus {
    OPEN,
    FILLED,
    CANCELLED,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq)]
#[sqlx(
    type_name = "\"polymarket\".\"user_transaction_type\"",
    rename_all = "lowercase"
)]
pub enum UserTransactionType {
    DEPOSIT,
    WITHDRAWAL,
    TRADE,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq)]
#[sqlx(
    type_name = "\"polymarket\".\"user_transaction_status\"",
    rename_all = "lowercase"
)]
pub enum UserTransactionStatus {
    PENDING,
    COMPLETED,
    FAILED,
}
