use capwat_error::{ext::NoContextResultExt, Result};
use capwat_macros::ConfigParts;
use capwat_utils::env;
use doku::Document;
use serde::Deserialize;

use super::{LoggingLoadError, LoggingStyle};
use crate::vars;

#[derive(Debug, Document, ConfigParts)]
#[config(attr(derive(Debug, Deserialize)))]
pub struct Global {
    /// **Environment variable**: `CAPWAT_LOGGING_STYLE`
    ///
    /// There are three styles to choose:
    /// - `compact` - compacts logs but it is readable enough
    /// - `full` - default formatter from [`tracing_subscriber`].
    /// - `pretty` - makes logs pretty
    /// - `json` - serializes logs into JSON data
    ///
    /// The default value is `compact`, if not set.
    #[doku(as = "String", example = "full")]
    pub style: LoggingStyle,

    /// **Environment variable**: `CAPWAT_LOGGING_GLOBAL_TARGETS` or `RUST_LOG`
    ///
    /// This property filters logging events with the use of directives.
    /// By default, it will filter events that have `info` level.
    ///
    /// You may refer on how directives work and parse and its examples by going to:
    /// https://docs.rs/tracing-subscriber/0.3.18/tracing_subscriber/filter/struct.EnvFilter.html
    ///
    /// The default value is a blank string, if not set.
    #[doku(example = "info")]
    pub targets: String,
}

impl Global {
    pub(crate) fn from_partial(partial: PartialGlobal) -> Result<Self, LoggingLoadError> {
        Ok(Self {
            style: partial.style.unwrap_or_default(),
            targets: partial.targets.unwrap_or_default(),
        })
    }
}

impl PartialGlobal {
    pub fn from_env() -> Result<Self, LoggingLoadError> {
        let style = env::var_opt_parsed::<LoggingStyle>(&vars::LOGGING_GLOBAL_STYLE)
            .change_context(LoggingLoadError)?;

        let targets = env::var_opt(&vars::LOGGING_GLOBAL_TARGETS)
            .change_context(LoggingLoadError)
            .transpose()
            .or_else(|| {
                env::var_opt("RUST_LOG")
                    .change_context(LoggingLoadError)
                    .transpose()
            })
            .transpose()?;

        Ok(Self { style, targets })
    }
}
