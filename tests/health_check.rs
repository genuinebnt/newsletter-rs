use std::sync::Arc;

use newsletter::{
    configuration::get_configuration,
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use sqlx::PgPool;
use tokio::net::TcpListener;

#[derive(Debug)]
pub struct TestApp {
    pub address: String,
    pub pool: Arc<PgPool>,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = get_subscriber("test".into(), "debug".into());
    init_subscriber(subscriber);
});

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let pool = Arc::new(
        PgPool::connect(&connection_string)
            .await
            .expect("Failed to connect to database"),
    );
    let app = newsletter::startup::router(pool.clone()).await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let _ = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        pool: pool.clone(),
    }
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=genuine&email=genuine.basilnt@gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name from subscriptions;")
        .fetch_one(&*app.pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "genuine.basilnt@gmail.com");
    assert_eq!(saved.name, "genuine");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=genuine", "missing the email"),
        ("email=genuine.basilnt@gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 422 Bad Request when the payload was {}.",
            error_message
        );
    }
}
