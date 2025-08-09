use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging(level: &str, format: crate::utils::config::LogFormat) {
    let log_level = match level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    match format {
        crate::utils::config::LogFormat::Json => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .with(tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(log_level.into()))
                .init();
        }
        crate::utils::config::LogFormat::Pretty => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().pretty())
                .with(tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(log_level.into()))
                .init();
        }
        crate::utils::config::LogFormat::Compact => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().compact())
                .with(tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(log_level.into()))
                .init();
        }
    }
}

