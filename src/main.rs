use newsletter::{
    configuration::get_configuration,
    startup::router,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::PgPool;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("newsletter".into(), "info".into());
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to get configuration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to postgres");

    let app = router(pool.into()).await;
    let listener = TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
