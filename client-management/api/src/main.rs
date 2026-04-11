mod config;
mod routes;
mod telemetry;

use config::Settings;

use anyhow::{Context, Result};
use axum::{Router, routing::get};
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use tokio::signal;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // Both `ring` and `aws-lc-rs` features are enabled on rustls (pulled in by
    // different transitive deps), so rustls cannot auto-detect a CryptoProvider.
    // Install the default (aws-lc-rs) before any TLS usage.
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install default CryptoProvider");

    let settings = Settings::load().context("Failed to load configuration")?;

    let telemetry_guard = telemetry::init_telemetry(&settings.tls)
        .context("Failed to initialise telemetry")?;

    let app = Router::new()
        .route("/health", get(routes::health))
        .layer(TraceLayer::new_for_http());

    let tls_config = RustlsConfig::from_pem_file(
        &settings.tls.certificate_path,
        &settings.tls.certificate_key_path,
    )
    .await
    .with_context(|| {
        format!(
            "Failed to load TLS config from {} / {}",
            settings.tls.certificate_path, settings.tls.certificate_key_path
        )
    })?;

    let addr = SocketAddr::from(([0, 0, 0, 0], settings.server.port));

    tracing::info!("Listening on {addr}");

    let server = axum_server::bind_rustls(addr, tls_config).serve(app.into_make_service());

    tokio::select! {
        result = server => {
            result.context("api server failed")?;
        }
        _ = shutdown_signal() => {
            tracing::info!("Shutdown signal received");
        }
    }

    telemetry_guard.shutdown();

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}
