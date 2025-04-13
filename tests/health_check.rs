use std::sync::Arc;

use newsletter::configuration::get_configuration;
use sqlx::PgPool;
use tokio::net::TcpListener;

struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to port");
    // retrive port assigned by the OS
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let configuration = get_configuration().expect("Failed to get configuration");
    let connection_string = configuration.database.connection_string();
    let pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to postgres");
    let router = newsletter::startup::router(pool.clone());

    // spawn the server in a seperate thread so tests can interact with it via http requests
    let _ = tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("Failed to spawn server")
    });

    TestApp {
        address,
        db_pool: pool,
    }
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health_check", app.address))
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
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions");

    assert_eq!(saved.email, "genuine.basilnt@gmail.com");
    assert_eq!(saved.name, "genuine");
}

#[tokio::test]
async fn subscribe_returns_a_422_when_data_is_missing() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=genuine", "missing email"),
        ("email=genuine.basilnt@gmail.com", "missing name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
}
