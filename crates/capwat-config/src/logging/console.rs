use std::{fmt::Display, str::FromStr};

use capwat_error::{ext::NoContextResultExt, Result};
use capwat_macros::ConfigParts;
use capwat_utils::env;
use doku::Document;
use serde::Deserialize;
use thiserror::Error;

use super::{LoggingLoadError, LoggingStyle};
use crate::vars;

#[derive(Debug, Document, ConfigParts)]
#[config(attr(derive(Debug, Deserialize)))]
pub struct Console {
    /// What stream should the logger log into?
    ///
    /// There are two options to choose from:
    /// - `stdout`
    /// - `stderr`
    ///
    /// The default value is `stderr`, if not set.
    #[doku(as = "String", example = "stderr")]
    pub stream: ConsoleStream,

    /// **Environment variable**: `CAPWAT_CONSOLE_LOGGING_STYLE`
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

    /// **Environment variable**: `CAPWAT_CONSOLE_LOGGING_TARGETS`
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

impl Console {
    pub(crate) fn from_partial(
        partial: PartialConsole,
        global: &super::Global,
    ) -> Result<Self, LoggingLoadError> {
        let stream = partial.stream.unwrap_or(ConsoleStream::Stderr);
        let style = partial.style.unwrap_or(global.style);
        let targets = partial.targets.unwrap_or_else(|| global.targets.clone());

        Ok(Self {
            stream,
            style,
            targets,
        })
    }
}

impl PartialConsole {
    pub fn from_env() -> Result<Self, LoggingLoadError> {
        let stream = env::var_opt_parsed::<ConsoleStream>(vars::CONSOLE_LOGGING_STREAM)
            .change_context(LoggingLoadError)?;

        let style = env::var_opt_parsed::<LoggingStyle>(vars::CONSOLE_LOGGING_STYLE)
            .change_context(LoggingLoadError)?;

        let targets =
            env::var_opt(vars::CONSOLE_LOGGING_TARGETS).change_context(LoggingLoadError)?;

        Ok(Self {
            stream,
            style,
            targets,
        })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsoleStream {
    Stdout,
    #[default]
    Stderr,
    TestWriter,
}

impl Display for ConsoleStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stderr => f.write_str("stdout"),
            Self::Stdout => f.write_str("stdout"),
            Self::TestWriter => f.write_str("test-writer"),
        }
    }
}

impl<'de> Deserialize<'de> for ConsoleStream {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = ConsoleStream;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("console stream")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                ConsoleStream::from_str(v).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[derive(Debug, Error)]
#[error("unknown {0:?} console stream")]
pub struct InvalidConsoleStream(String);

impl FromStr for ConsoleStream {
    type Err = InvalidConsoleStream;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "stdout" => Ok(Self::Stdout),
            "stderr" => Ok(Self::Stderr),
            _ => Err(InvalidConsoleStream(s)),
        }
    }
}
