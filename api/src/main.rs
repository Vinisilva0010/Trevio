use std::process::ExitCode;
use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{error, info};
use trevio_api::app::{AppState, build_router};
use trevio_api::config::{Config, READY_TIMEOUT};
use trevio_api::telemetry;

#[tokio::main]
async fn main() -> ExitCode {
    let config = match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            // Logging is not initialised yet (it needs the validated level),
            // so a plain stderr line is the only option. The error text never
            // contains configuration values.
            #[allow(clippy::print_stderr)]
            {
                eprintln!("trevio-api: invalid configuration: {err}");
            }
            return ExitCode::from(2);
        }
    };
    telemetry::init(config.log_level);

    match run(config).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!(error = %err, "server stopped with an error");
            ExitCode::FAILURE
        }
    }
}

async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // Lazy pool: the process starts (and answers /health) even if Postgres is
    // down; /ready reports the truth. No migrations run here, by design.
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(READY_TIMEOUT)
        .idle_timeout(Some(Duration::from_secs(300)))
        .connect_lazy(config.database_url.expose())?;

    let listener = TcpListener::bind(config.bind_addr).await?;
    info!(addr = %config.bind_addr, "listening");

    axum::serve(listener, build_router(AppState { pool }))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if signal::ctrl_c().await.is_err() {
            // Cannot install the handler: never resolve, rely on SIGTERM.
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    info!("shutdown signal received");
}
