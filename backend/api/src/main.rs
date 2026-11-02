use api::{AppState, config::Config, telemetry};
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    telemetry::init(config.log_format);
    tracing::debug!(?config, "config loaded");

    let db = PgPoolOptions::new()
        .max_connections(config.db.max_connections)
        .acquire_timeout(config.db.acquire_timeout)
        .connect(config.db.url.expose_secret())
        .await?;
    // Embedded from `migrations/`. Safe while only one API instance runs.
    sqlx::migrate!().run(&db).await?;

    let app = api::router(AppState { db }, &config.http);

    let listener = TcpListener::bind(config.addr).await?;
    tracing::info!(addr = %config.addr, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
