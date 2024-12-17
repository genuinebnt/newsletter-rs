use axum::extract::Request;
use axum::response::IntoResponse;
use reqwest::StatusCode;

pub async fn health_check(_req: Request) -> impl IntoResponse {
    StatusCode::OK
}
