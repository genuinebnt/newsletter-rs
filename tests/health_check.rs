use newsletter::configuration::{DatabaseSettings, get_configuration};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tokio::net::TcpListener;
use uuid::Uuid;

struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to create database");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database");
    connection_pool
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to port");
    // retrive port assigned by the OS
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut configuration = get_configuration().expect("Failed to get configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let pool = configure_database(&configuration.database).await;
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
