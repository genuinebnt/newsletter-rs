use newsletter::{configuration, startup};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = configuration::get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Failed to bind to address");
    let pool = PgPool::connect(&connection_string).await.unwrap();
    let router = startup::router(pool);
    axum::serve(listener, router)
        .await
        .expect("Failed to create server");
    Ok(())
}
