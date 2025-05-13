use axum::{http::StatusCode, response::Response};
use serde::{Deserialize, Serialize};

pub type ReturnType = (StatusCode, Response);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginationRequestQuery {
    pub page: i64,
    pub page_size: i64,
}
