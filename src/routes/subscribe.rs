use axum::response::IntoResponse;
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
struct FormData {
    email: String,
    name: String,
}

async fn subscribe() -> impl IntoResponse {
    StatusCode::OK
}
