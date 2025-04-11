use tokio::net::TcpListener;

async fn spawn_app() -> Result<(), std::io::Error> {
    let router = newsletter::startup::router();
    let listener = TcpListener::bind("127.0.0.1:8000")
        .await
        .expect("Failed to bind to port");
    let _ = tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("Failed to spawn server")
    });

    Ok(())
}

#[tokio::test]
async fn health_check_works() {
    spawn_app().await.expect("Failed to spawn our app");

    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
