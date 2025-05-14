use axum::{http::StatusCode, response::Response};
use serde::{Deserialize, Serialize};

pub type ReturnType = (StatusCode, Response);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginationRequestQuery {
    pub page: u64,
    #[serde(rename = "pageSize")]
    pub page_size: u64,
}
