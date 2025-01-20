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

#[tracing::instrument(name = "Adding a new subscriber", skip(form, state), fields(request_id = %Uuid::new_v4(),subscriber_email = %form.email, subscriber_name = %form.name))]
pub async fn subscribe(
    State(state): State<Arc<AppState>>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    match insert_subscriber(State(state), Form(form)).await {
        Ok(_) => (StatusCode::OK, "Thanks for subscribing"),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "There was a problem, please try again later",
        ),
    }
}

#[tracing::instrument(name = "Saving new subscriber details in database", skip(form, state))]
pub async fn insert_subscriber(
    State(state): State<Arc<AppState>>,
    Form(form): Form<FormData>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        chrono::Utc::now(),
    )
    .execute(&*state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
