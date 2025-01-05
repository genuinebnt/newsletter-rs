use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Form};
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::types::{chrono, Uuid};

use crate::startup::AppState;

#[derive(Debug, Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    State(state): State<Arc<AppState>>,
    Form(form): Form<FormData>
) -> impl IntoResponse {
    let uuid = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query!(
        r#"INSERT INTO subscriptions(id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        uuid,
        form.email,
        form.name,
        now
    )
    .execute(&*state.pool)
    .await
    .expect("Failed to execute query");

    StatusCode::OK
}
