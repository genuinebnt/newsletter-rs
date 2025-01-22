use newsletter::{
    configuration::get_configuration,
    startup::router,
    telemetry::{get_subscriber, init_subscriber},
};
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("newsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to get configuration");
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(configuration.database.connection_string().expose_secret())
        .expect("Failed to connect to postgres");

    let app = router(pool.into()).await;
    let listener = TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
