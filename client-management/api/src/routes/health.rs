use axum::http::StatusCode;
use tracing::instrument;

#[instrument]
pub async fn health() -> StatusCode {
    StatusCode::OK
}
