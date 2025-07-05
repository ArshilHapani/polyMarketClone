use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::state::AppState;

pub mod cancel_order;
pub mod create_order;
pub mod get_user_orders;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/get", get(get_user_orders::get_user_orders))
        .route("/create", post(create_order::create_order))
        .route("/get/{id}", get(get_user_orders::get_user_orders_by_market))
        .route("/cancel/{id}", patch(cancel_order::cancel_order))
}
