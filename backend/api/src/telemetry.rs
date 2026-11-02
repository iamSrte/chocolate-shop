//! Tracing setup. Add exporters (OpenTelemetry, Sentry) here as extra layers when needed.

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::LogFormat;

/// Installs the global subscriber. Levels come from `RUST_LOG` (default `info`).
pub fn init(log_format: LogFormat) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    let registry = tracing_subscriber::registry().with(filter);
    let format = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_target(true);

    match log_format {
        LogFormat::Pretty => registry.with(format).init(),
        LogFormat::Json => registry
            .with(
                format
                    .json()
                    .flatten_event(true)
                    .with_current_span(true)
                    .with_span_list(false),
            )
            .init(),
    }
}
