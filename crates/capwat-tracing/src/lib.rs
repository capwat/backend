use capwat_config::{
    logging::{ConsoleStream, LoggingStyle},
    vars,
};
use capwat_error::ext::ResultExt;
use capwat_error::Result;
use thiserror::Error;
use tracing::{level_filters::LevelFilter, warn};
use tracing_appender::non_blocking::WorkerGuard as FileLayerGuard;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

pub mod logging;

#[derive(Debug, Error)]
#[error("Failed to initialize tracing")]
pub struct TracingInitError;

#[allow(unused)]
pub struct TracingInitGuard {
    file_guard: Option<FileLayerGuard>,
}

pub fn init(config: &capwat_config::Logging) -> Result<TracingInitGuard, TracingInitError> {
    let console = self::logging::console_layer(&config.console);
    let (file, file_guard) = self::logging::file_layer(&config.file)?;

    let registry = tracing_subscriber::Registry::default()
        .with(console)
        .with(file);

    tracing::subscriber::set_global_default(registry)
        .change_context(TracingInitError)
        .attach_printable("already initialized tracing")?;

    if std::env::var("RUST_LOG").is_ok() && std::env::var(vars::LOGGING_GLOBAL_TARGETS).is_ok() {
        warn!("Both `RUST_LOG` and `{}` are set, please pick one of them to determine the global logging targets", vars::LOGGING_GLOBAL_TARGETS);
    }

    Ok(TracingInitGuard { file_guard })
}

pub fn init_for_tests() {
    let maker = self::logging::LoggingStreamMaker::new(ConsoleStream::TestWriter);
    let console = self::logging::common_layer(
        maker,
        false,
        &LoggingStyle::Full,
        &capwat_utils::env::var("RUST_LOG").ok().unwrap_or_default(),
    );

    let registry = tracing_subscriber::Registry::default().with(console);
    tracing::subscriber::set_global_default(registry)
        .change_context(TracingInitError)
        .unwrap();
}

fn make_env_filter(targets: &str) -> EnvFilter {
    let default_level = if cfg!(release) {
        LevelFilter::INFO
    } else {
        LevelFilter::DEBUG
    };

    EnvFilter::builder()
        .with_default_directive(default_level.into())
        .parse_lossy(targets)
}
