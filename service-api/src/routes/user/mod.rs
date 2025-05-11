use axum::{Router, routing::post};

use crate::state::AppState;
use login::login;

pub mod login;

pub fn router() -> Router<AppState> {
    Router::new().route("/login", post(login))
}
