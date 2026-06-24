use crate::Directory;
use crate::infrastructure::config::AppConfigure;
use crate::infrastructure::environment::Environment;
use tracing::subscriber::set_global_default;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter, Registry,
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

    // Create a log directory.
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::HOURLY)
        .filename_prefix("server")
        .filename_suffix("log")
        .build(logs_dir)
        .expect("Creating a log file writer failed.");

    let (writer, guard) = tracing_appender::non_blocking(file_appender);

    // Set up log system.
    // When the server runs in a development environment, enable more settings.
    if env.is_development() {
        let filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

        let file = fmt::layer()
            .json()
            .with_writer(writer.clone())
            .with_ansi(false)
            .with_span_events(FmtSpan::CLOSE)
            .with_current_span(false)
            .with_thread_names(false);

        let stdout = fmt::layer()
            .with_writer(std::io::stderr)
            .with_ansi(true)
            .with_level(true)
            .with_target(true)
            .with_thread_ids(false)
            .with_thread_names(true);

        let subscriber = Registry::default().with(filter).with(stdout).with(file);

        set_global_default(subscriber)?;
        tracing::info!("Log system initialization complete.");
        return Ok(guard);
    }

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    let file = fmt::layer()
        .json()
        .with_writer(writer)
        .with_ansi(false)
        .with_span_events(FmtSpan::CLOSE)
        .with_current_span(false)
        .with_thread_names(true);

    let stdout = fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true);

    let subscriber = Registry::default().with(filter).with(stdout).with(file);

    set_global_default(subscriber)?;
    tracing::info!("Log system initialization complete.");
    Ok(guard)
}
