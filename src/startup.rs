use crate::routes::{health_check::health_check, subscribe::subscribe};
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use std::sync::Arc;

pub struct AppState {
    pub pool: PgPool,
}

pub fn router(pool: PgPool) -> Router {
    let app_state = Arc::new(AppState { pool: pool });

    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(app_state)
}
