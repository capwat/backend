use capwat_error::{
    ext::{NoContextResultExt, ResultExt},
    Result,
};
use capwat_macros::ConfigParts;
use capwat_utils::env;
use doku::Document;
use serde::Deserialize;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

use super::{LoggingLoadError, LoggingStyle};
use crate::vars;

#[derive(Debug, Document, ConfigParts)]
#[config(attr(derive(Debug, Deserialize)))]
#[config(attr(serde(rename_all = "kebab-case")))]
pub struct File {
    /// **Environment variable**: `CAPWAT_FILE_LOGGING_ENABLED`
    ///
    /// Whether file logging is enabled or not.
    ///
    /// The default value is `false`, if not set.
    #[doku(example = "false")]
    pub enabled: bool,

    /// **Environment variable**: `CAPWAT_FILE_LOGGING_PATH`
    ///
    /// A directory where all the logs should be placed in.
    ///
    /// The default value is `<current_directory>/logs`, if not set.
    #[doku(as = "String", example = "/etc/capwat/logs")]
    pub output: PathBuf,

    /// **Environment variable**: `CAPWAT_FILE_LOGGING_ROTATION_INTERVAL`
    ///
    /// There are four choices to choose:
    /// - `daily`
    /// - `hourly`
    /// - `minutely`
    /// - `never`
    ///
    /// The default value is `never`, if not set.
    #[doku(as = "String", example = "never")]
    pub rotation_interval: FileRotationInterval,

    /// **Environment variable**: `CAPWAT_FILE_LOGGING_STYLE`
    ///
    /// There are three styles to choose:
    /// - `compact` - compacts logs but it is readable enough
    /// - `full` - default formatter from [`tracing_subscriber`].
    /// - `pretty` - makes logs pretty
    /// - `json` - serializes logs into JSON data
    ///
    /// The default value is `json`, if not set. Global logging
    /// configuration will not be inherited in this field.
    #[doku(as = "String", example = "full")]
    pub style: LoggingStyle,

    /// **Environment variable**: `CAPWAT_FILE_LOGGING_TARGETS`
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

impl File {
    pub(crate) fn from_partial(
        partial: PartialFile,
        global: &super::Global,
    ) -> Result<Self, LoggingLoadError> {
        let output = partial.output.map(|v| Ok(v)).unwrap_or_else(|| {
            std::env::current_dir()
                .change_context(LoggingLoadError)
                .attach_printable("cannot get current directory to save file logs")
                .map(|v| v.join("logs"))
        })?;

        let style = partial.style.unwrap_or(LoggingStyle::JSON);
        let targets = partial.targets.unwrap_or_else(|| global.targets.clone());

        Ok(Self {
            enabled: partial.enabled.unwrap_or(false),
            output,
            rotation_interval: partial.rotation_interval.unwrap_or_default(),
            style,
            targets,
        })
    }
}

impl PartialFile {
    pub fn from_env() -> Result<Self, LoggingLoadError> {
        let enabled =
            env::var_opt_parsed(&vars::FILE_LOGGING_ENABLED).change_context(LoggingLoadError)?;

        let output =
            env::var_opt_parsed(&vars::FILE_LOGGING_OUTPUT).change_context(LoggingLoadError)?;

        let rotation_interval = env::var_opt_parsed(&vars::FILE_LOGGING_ROTATION_INTERVAL)
            .change_context(LoggingLoadError)?;

        let style = env::var_opt_parsed::<LoggingStyle>(&vars::FILE_LOGGING_STYLE)
            .change_context(LoggingLoadError)?;

        let targets = env::var_opt(&vars::FILE_LOGGING_TARGETS).change_context(LoggingLoadError)?;

        Ok(Self {
            enabled,
            output,
            rotation_interval,
            style,
            targets,
        })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileRotationInterval {
    Hourly,
    Minutely,
    Daily,
    #[default]
    Never,
}

impl Display for FileRotationInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hourly => f.write_str("hourly"),
            Self::Minutely => f.write_str("minutely"),
            Self::Daily => f.write_str("daily"),
            Self::Never => f.write_str("never"),
        }
    }
}

impl<'de> Deserialize<'de> for FileRotationInterval {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = FileRotationInterval;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("file rotation interval")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                FileRotationInterval::from_str(v).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[derive(Debug, Error)]
#[error("unknown {0:?} file rotation interval")]
pub struct InvalidFileRotationInterval(String);

impl FromStr for FileRotationInterval {
    type Err = InvalidFileRotationInterval;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "hourly" => Ok(Self::Hourly),
            "minutely" => Ok(Self::Minutely),
            "daily" => Ok(Self::Daily),
            "never" => Ok(Self::Never),
            _ => Err(InvalidFileRotationInterval(s)),
        }
    }
}
