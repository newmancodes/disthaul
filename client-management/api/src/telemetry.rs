use std::path::Path;

use anyhow::{Context, Result};
use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::LogExporter;
use opentelemetry_otlp::MetricExporter;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_otlp::WithTonicConfig;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tonic::transport::{Certificate, ClientTlsConfig};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Holds the OTEL providers so they can be shut down gracefully.
pub struct TelemetryGuard {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
    logger_provider: SdkLoggerProvider,
}

impl TelemetryGuard {
    /// Shuts down all OTEL providers, flushing any remaining telemetry.
    pub fn shutdown(self) {
        if let Err(err) = self.tracer_provider.shutdown() {
            eprintln!("Failed to shut down tracer provider: {err}");
        }
        if let Err(err) = self.meter_provider.shutdown() {
            eprintln!("Failed to shut down meter provider: {err}");
        }
        if let Err(err) = self.logger_provider.shutdown() {
            eprintln!("Failed to shut down logger provider: {err}");
        }
    }
}

/// Initialises OpenTelemetry telemetry (traces, metrics, logs) using the OTLP
/// exporter, driven by standard OTEL environment variables that Aspire injects
/// via `.WithOtlpExporter()`:
///
/// - `OTEL_EXPORTER_OTLP_ENDPOINT`
/// - `OTEL_EXPORTER_OTLP_PROTOCOL`
/// - `OTEL_EXPORTER_OTLP_HEADERS`
/// - `OTEL_SERVICE_NAME`
/// - `OTEL_RESOURCE_ATTRIBUTES`
///
/// `tls_cert_path` is the path to the PEM certificate file (provided by Aspire
/// via `TLS__CERTIFICATE_PATH`). This certificate is added as a trusted CA root
/// so tonic can connect to Aspire's HTTPS gRPC OTLP endpoint, which uses a
/// self-signed / locally-issued certificate.
///
/// When `OTEL_EXPORTER_OTLP_ENDPOINT` is not set (e.g. running outside Aspire),
/// OTLP export still initialises but targets the SDK default endpoint
/// (`https://localhost:4317`). The `fmt` layer always provides local stdout
/// logging regardless.
pub fn init_telemetry(tls_cert_path: &str) -> Result<TelemetryGuard> {
    let tls_config = build_tls_config(tls_cert_path)?;

    // --- Traces ---
    let span_exporter = SpanExporter::builder()
        .with_tonic()
        .with_tls_config(tls_config.clone())
        .build()
        .context("Failed to create OTLP span exporter")?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .build();

    global::set_tracer_provider(tracer_provider.clone());

    // --- Metrics ---
    let metric_exporter = MetricExporter::builder()
        .with_tonic()
        .with_tls_config(tls_config.clone())
        .build()
        .context("Failed to create OTLP metric exporter")?;

    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .build();

    global::set_meter_provider(meter_provider.clone());

    // --- Logs ---
    let log_exporter = LogExporter::builder()
        .with_tonic()
        .with_tls_config(tls_config)
        .build()
        .context("Failed to create OTLP log exporter")?;

    let logger_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .build();

    // --- Tracing subscriber ---
    // 1. OpenTelemetry tracing layer: converts `tracing` spans into OTEL spans
    let otel_trace_layer =
        tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("client-management-api"));

    // 2. OpenTelemetry log bridge: converts `tracing` events into OTEL log records
    let otel_log_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    // 3. fmt layer: stdout logging for local development
    let fmt_layer = tracing_subscriber::fmt::layer();

    // 4. EnvFilter: respects RUST_LOG, defaults to `info`
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(otel_trace_layer)
        .with(otel_log_layer)
        .with(fmt_layer)
        .init();

    tracing::info!("Telemetry initialised");

    Ok(TelemetryGuard {
        tracer_provider,
        meter_provider,
        logger_provider,
    })
}

/// Builds a [`ClientTlsConfig`] that trusts the certificate at `cert_path`.
///
/// Aspire provides a self-signed / locally-issued PEM certificate for both the
/// API server and the OTLP collector endpoint. Adding this certificate as a
/// trusted CA root allows tonic/rustls to complete the TLS handshake with the
/// Aspire dashboard's gRPC OTLP endpoint.
fn build_tls_config(cert_path: &str) -> Result<ClientTlsConfig> {
    let pem = std::fs::read(Path::new(cert_path))
        .with_context(|| format!("Failed to read TLS certificate from {cert_path}"))?;

    let tls_config = ClientTlsConfig::new().ca_certificate(Certificate::from_pem(pem));

    Ok(tls_config)
}
