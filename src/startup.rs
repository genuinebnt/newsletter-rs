use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use sqlx::PgConnection;

use crate::routes::{health_check, subscribe};

struct AppState {
    connection: PgConnection,
}

pub async fn router(connection: PgConnection) -> Router {
    let state = Arc::new(AppState { connection });

    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", get(subscribe))
        .with_state(state)
}
