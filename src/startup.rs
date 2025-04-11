use axum::{Router, routing::get};

use crate::health_check::health_check;

pub fn router() -> Router {
    Router::new().route("/health_check", get(health_check))
}
