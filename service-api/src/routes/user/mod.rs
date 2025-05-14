use axum::{Router, routing::get};

use crate::state::AppState;

pub mod orders;
pub mod profile;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/orders", orders::router())
        .route("/profile", get(profile::get_profile))
}
