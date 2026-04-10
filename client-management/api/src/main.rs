mod config;
mod routes;

use config::Settings;

use anyhow::{Context, Result};
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let settings = Settings::load()
        .context("Failed to load configuration")?;

    let app = Router::new().route("/health", get(routes::health));

    let addr = SocketAddr::from(([0, 0, 0, 0], settings.server.port));
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind to {addr}"))?;

    axum::serve(listener, app)
        .await
        .context("api server failed")?;

    Ok(())
}
