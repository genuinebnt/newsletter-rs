use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;

use crate::routes::{health_check::health_check, subscribe::subscribe};

pub struct AppState {
    pub pool: Arc<PgPool>,
}

pub fn router(pool: PgPool) -> Router {
    let app_state = Arc::new(AppState {
        pool: Arc::new(pool),
    });

    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(app_state)
}
