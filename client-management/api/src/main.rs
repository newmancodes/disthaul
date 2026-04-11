mod config;
mod routes;

use config::Settings;

use anyhow::{Context, Result};
use axum::{Router, routing::get};
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let settings = Settings::load().context("Failed to load configuration")?;

    let app = Router::new().route("/health", get(routes::health));

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

    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
        .context("api server failed")?;

    Ok(())
}
