use crate::Directory;
use crate::infrastructure::config::AppConfigure;
use crate::infrastructure::environment::Environment;
use tracing::subscriber::set_global_default;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter, Layer,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
};

pub fn init_log(
    directory: &Directory,
    configure: &AppConfigure,
    env: Environment,
) -> Result<tracing_appender::non_blocking::WorkerGuard, Box<dyn std::error::Error>> {
    let logs_dir = directory.log.clone();
    let config = &configure.log;

    // Create a log file writer.
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::HOURLY)
        .filename_prefix("server")
        .filename_suffix("log")
        .build(logs_dir)
        .expect("Creating a log file writer failed.");

    let (non_blocking_writer, guard) = tracing_appender::non_blocking(file_appender);

    // Create an environment filter.
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    // Console layer: pretty in development, compact JSON in production.
    let console_layer = if env.is_development() {
        fmt::layer()
            .with_writer(std::io::stderr)
            .with_ansi(true)
            .with_level(true)
            .with_target(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .boxed()
    } else {
        fmt::layer()
            .json()
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .with_level(true)
            .with_target(true)
            .with_thread_ids(true)
            .boxed()
    };

    // File layer: always JSON.
    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking_writer)
        .with_ansi(false)
        .with_span_events(FmtSpan::CLOSE)
        .with_current_span(false)
        .with_thread_names(false);

    // Build subscribers.
    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer);

    set_global_default(subscriber)?;

    tracing::info!("Log system initialization complete.");

    Ok(guard)
}
