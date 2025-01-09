use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Form};
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::types::{chrono, Uuid};
use tracing::Instrument;

use crate::startup::AppState;

#[derive(Debug, Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    State(state): State<Arc<AppState>>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!("Adding a new subscriber.", %request_id, subscriber_email = %form.email, subscriber_name = %form.name);
    let _request_span_guard = request_span.enter();
    let query_span = tracing::info_span!("Saving new subscriber details in database");

    let uuid = Uuid::new_v4();
    let now = chrono::Utc::now();
    match sqlx::query!(
        r#"INSERT INTO subscriptions(id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        uuid,
        form.email,
        form.name,
        now
    )
    .execute(&*state.pool)
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request id {} - New subscriber details have been saved",
                request_id
            );
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!(
                "request id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
