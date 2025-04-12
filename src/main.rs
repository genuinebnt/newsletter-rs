use newsletter::{configuration, startup};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = configuration::get_configuration().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Failed to bind to address");
    let router = startup::router();
    axum::serve(listener, router).await.expect("Failed to create server");
    Ok(())
}
