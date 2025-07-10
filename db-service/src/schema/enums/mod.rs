use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"market_status\"")]
#[sqlx(rename_all = "lowercase")]
pub enum MarketStatus {
    #[default]
    #[serde(rename = "open")]
    OPEN,
    #[serde(rename = "closed")]
    CLOSED,
    #[serde(rename = "settled")]
    SETTLED,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy, Eq, Hash)]
#[sqlx(type_name = "\"polymarket\".\"outcome\"")]
#[sqlx(rename_all = "lowercase")]
pub enum Outcome {
    #[serde(rename = "yes")]
    YES,
    #[serde(rename = "no")]
    NO,
    #[default]
    #[serde(rename = "unspecified")]
    UNSPECIFIED,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy, Eq, Hash)]
#[sqlx(type_name = "\"polymarket\".\"order_side\"")]
#[sqlx(rename_all = "lowercase")]
pub enum OrderSide {
    #[default]
    #[serde(rename = "buy")]
    BUY, // bids
    #[serde(rename = "sell")]
    SELL, // asks
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"order_status\"")]
#[sqlx(rename_all = "lowercase")]
pub enum OrderStatus {
    #[default]
    #[serde(rename = "open")]
    OPEN,
    #[serde(rename = "filled")]
    FILLED,
    #[serde(rename = "cancelled")]
    CANCELLED,
    #[serde(rename = "expired")]
    EXPIRED,
    #[serde(rename = "unspecified")]
    UNSPECIFIED,
    #[sqlx(rename = "pending_update")]
    PendingUpdate,
    #[sqlx(rename = "pending_cancel")]
    PendingCancel,
    // NOT USED!!!! and DON'T USE IT
    #[sqlx(rename = "partial_fill")]
    PartialFill,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"user_transaction_type\"")]
#[sqlx(rename_all = "lowercase")]
pub enum UserTransactionType {
    #[default]
    #[serde(rename = "deposit")]
    DEPOSIT,
    #[serde(rename = "withdrawal")]
    WITHDRAWAL,
    #[serde(rename = "trade")]
    TRADE,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"user_transaction_status\"")]
#[sqlx(rename_all = "lowercase")]
pub enum UserTransactionStatus {
    #[default]
    #[serde(rename = "pending")]
    PENDING,
    #[serde(rename = "completed")]
    COMPLETED,
    #[serde(rename = "failed")]
    FAILED,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq, Default, Copy)]
#[sqlx(type_name = "\"polymarket\".\"order_type\"")]
#[sqlx(rename_all = "lowercase")]
pub enum OrderType {
    #[default]
    #[serde(rename = "limit")]
    LIMIT,
    #[serde(rename = "market")]
    MARKET,
    #[serde(rename = "stop_loss")]
    StopLoss,
    #[serde(rename = "take_profit")]
    TakeProfit,
}
