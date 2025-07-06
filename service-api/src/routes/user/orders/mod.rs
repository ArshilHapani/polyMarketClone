use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use crate::state::AppState;

pub mod cancel_order;
pub mod create_order;
pub mod get_orders;
pub mod update_order;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/get", get(get_orders::get_user_orders))
        .route("/create", post(create_order::create_order))
        .route("/get/{id}", get(get_orders::get_user_orders_by_market))
        .route("/cancel/{id}", delete(cancel_order::cancel_order))
        .route("/update", patch(update_order::update_order))
}
