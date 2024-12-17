use axum::{response::IntoResponse, Form};
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(Form(form): Form<FormData>) -> impl IntoResponse {
    StatusCode::OK
}
