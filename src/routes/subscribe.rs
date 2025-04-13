use std::sync::Arc;

use axum::{Form, extract::State, response::IntoResponse};
use chrono::Utc;
use reqwest::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::startup::AppState;

#[derive(Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

pub async fn subscribe(
    State(state): State<Arc<AppState>>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(&*state.pool)
    .await
    .unwrap();
    StatusCode::OK
}
