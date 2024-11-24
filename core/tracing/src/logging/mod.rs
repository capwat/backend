mod formatter;
mod writer;

pub(crate) use self::formatter::Formatter;
pub(crate) use self::writer::LoggingStreamMaker;

use crate::{make_env_filter, TracingInitError};

use capwat_config::logging::{FileRotationInterval, LoggingStyle};
use capwat_error::ext::ResultExt;
use capwat_error::Result;
use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard as FileLayerGuard;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

// TODO: Add support for log compression by forking tracing_appender
pub fn file_layer<S>(
    config: &capwat_config::logging::File,
) -> Result<(Option<impl Layer<S>>, Option<FileLayerGuard>), TracingInitError>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    if !config.enabled {
        return Ok((None, None));
    }

    let rotation = match config.rotation_interval {
        FileRotationInterval::Hourly => tracing_appender::rolling::Rotation::HOURLY,
        FileRotationInterval::Minutely => tracing_appender::rolling::Rotation::MINUTELY,
        FileRotationInterval::Daily => tracing_appender::rolling::Rotation::DAILY,
        FileRotationInterval::Never => tracing_appender::rolling::Rotation::NEVER,
    };

    let appender = tracing_appender::rolling::Builder::new()
        .rotation(rotation)
        .filename_suffix("log")
        .build(&config.output)
        .change_context(TracingInitError)
        .attach_printable("could not initialize file logging")
        .attach_printable("suggestion: try to make that path from `logging.file.path` exists and it is a directory")?;

    let (writer, guard) = tracing_appender::non_blocking(appender);
    let layer = common_layer(writer, false, &config.style, &config.targets);

    Ok((Some(layer), Some(guard)))
}

pub fn console_layer<S>(config: &capwat_config::logging::Console) -> impl Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let maker = LoggingStreamMaker::new(config.stream);
    let ansi = maker.supports_color();
    common_layer(maker, ansi, &config.style, &config.targets)
}

pub(crate) fn common_layer<S>(
    maker: impl for<'w> MakeWriter<'w> + Sync + Send + 'static,
    ansi: bool,
    style: &LoggingStyle,
    targets: &str,
) -> impl Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    if let Some(formatter) = Formatter::from_style(&style, ansi) {
        tracing_subscriber::fmt::layer()
            .with_ansi(ansi)
            .event_format(formatter)
            .with_writer(maker)
            .with_filter(make_env_filter(targets))
            .boxed()
    } else {
        debug_assert_eq!(*style, LoggingStyle::JSON);
        json_subscriber::fmt::layer()
            .with_flat_span_list(true)
            .with_writer(maker)
            .with_filter(make_env_filter(&targets))
            .boxed()
    }
}
