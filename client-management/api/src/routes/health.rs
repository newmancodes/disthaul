use axum::http::StatusCode;
use opentelemetry::{global, KeyValue};
use tracing::instrument;

#[instrument]
pub async fn health() -> StatusCode {
    let meter = global::meter("client-management-api");
    let counter = meter.u64_counter("health_check.count").build();
    counter.add(1, &[KeyValue::new("endpoint", "/health")]);
    tracing::info!("Health check");
    
    StatusCode::OK
}
