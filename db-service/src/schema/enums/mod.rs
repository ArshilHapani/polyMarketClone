use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"market_status\"")]
#[sqlx(rename_all = "lowercase")]
pub enum MarketStatus {
    #[default]
    OPEN,
    CLOSED,
    SETTLED,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"outcome\"")]
#[sqlx(rename_all = "lowercase")]
pub enum Outcome {
    YES,
    NO,
    #[default]
    UNSPECIFIED,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"order_side\"")]
#[sqlx(rename_all = "lowercase")]
pub enum OrderSide {
    #[default]
    BUY,
    SELL,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"order_status\"")]
#[sqlx(rename_all = "lowercase")]
pub enum OrderStatus {
    #[default]
    OPEN,
    FILLED,
    CANCELLED,
    EXPIRED,
    UNSPECIFIED,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"user_transaction_type\"")]
#[sqlx(rename_all = "lowercase")]
pub enum UserTransactionType {
    #[default]
    DEPOSIT,
    WITHDRAWAL,
    TRADE,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"user_transaction_status\"")]
#[sqlx(rename_all = "lowercase")]
pub enum UserTransactionStatus {
    #[default]
    PENDING,
    COMPLETED,
    FAILED,
}
