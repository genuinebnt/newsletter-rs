use newsletter::configuration::{self, get_configuration};
use sqlx::{Connection, PgConnection};
use tokio::net::TcpListener;

async fn spawn_app() -> String {
    let router = newsletter::startup::router();

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to port");
    // retrive port assigned by the OS
    let port = listener.local_addr().unwrap().port();
    // spawn the server in a seperate thread so tests can interact with it via http requests
    let _ = tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("Failed to spawn server")
    });

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app().await;
    let configuration = get_configuration().expect("Failed to get configuration");
    let connection_string = configuration.database.connection_string();
    let connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to postgres");
    let client = reqwest::Client::new();

    let body = "name=genuine&email=genuine.basilnt@gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=genuine", "missing email"),
        ("email=genuine.basilnt@gmail.com", "missing name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
}
