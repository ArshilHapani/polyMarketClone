use axum::{Router, routing::get};

use crate::state::AppState;

pub mod profile;

pub fn router() -> Router<AppState> {
    Router::new().route("/profile", get(profile::get_profile))
}
