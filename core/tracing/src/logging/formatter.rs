use capwat_config::logging::LoggingStyle;

use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::{Compact, Format, Full, Pretty, Writer};
use tracing_subscriber::fmt::time::ChronoUtc;
use tracing_subscriber::fmt::{self, FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

pub enum Formatter {
    Full(Format<Full, ChronoUtc>),
    Pretty(Format<Pretty, ChronoUtc>),
    Compact(Format<Compact, ChronoUtc>),
}

impl Formatter {
    pub fn from_style(style: &LoggingStyle, ansi: bool) -> Option<Self> {
        let default = fmt::format().with_timer(ChronoUtc::new("%Y-%m-%dT%H:%m:%S.%f".to_string()));
        match style {
            LoggingStyle::Compact => Some(Self::Compact(default.compact().with_ansi(ansi))),
            LoggingStyle::Full => Some(Self::Full(default.with_ansi(ansi))),
            LoggingStyle::Pretty => Some(Self::Pretty(default.pretty().with_ansi(ansi))),
            LoggingStyle::JSON => None,
        }
    }
}

impl<S, N> FormatEvent<S, N> for Formatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        match self {
            Formatter::Full(fmt) => fmt.format_event(ctx, writer, event),
            Formatter::Pretty(fmt) => fmt.format_event(ctx, writer, event),
            Formatter::Compact(fmt) => fmt.format_event(ctx, writer, event),
        }
    }
}
