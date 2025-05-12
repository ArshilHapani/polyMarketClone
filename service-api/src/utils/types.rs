use axum::{http::StatusCode, response::Response};

pub type ReturnType = (StatusCode, Response);
