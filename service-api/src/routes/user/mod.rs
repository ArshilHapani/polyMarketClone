use axum::{Router, routing::post};

use crate::state::AppState;
use login::oauth_login;

pub mod login;

pub fn router() -> Router<AppState> {
    Router::new().route("/login", post(oauth_login))
}
