use crate::routes::{health_check, subscribe};
use axum::routing::{get, post};
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: Arc<PgPool>,
}

pub async fn router(pool: Arc<PgPool>) -> Router {
    let state = Arc::new(AppState { pool });
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
