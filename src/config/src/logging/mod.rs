use capwat_error::Result;
use capwat_macros::ConfigParts;
use doku::Document;
use serde::Deserialize;
use thiserror::Error;

mod console;
mod file;
mod global;
mod style;

pub use self::console::{Console, ConsoleStream};
pub use self::file::{File, FileRotationInterval};
pub use self::global::Global;
pub use self::style::{InvalidLoggingStyle, LoggingStyle};

#[derive(Debug, Document, ConfigParts)]
#[config(attr(derive(Debug, Default, Deserialize)))]
#[config(attr(serde(default)))]
pub struct Logging {
    /// This field serves as a default configuration for all
    /// logging services available from console logging to file
    /// logging if one field that is related from global
    /// configuration is not present.
    ///
    /// **Example**:
    /// ```toml,no-run
    /// [logging.global]
    /// style = "pretty"
    ///
    /// # Since style is not specified in the console
    /// # logging configuration so we can get the global
    /// # configuration and set `style` in `logging.console`
    /// to `pretty` since `pretty` is set from the `logging.global`.
    /// [logging.console]
    /// ```
    #[config(as_struct, as_type = "self::global::PartialGlobal")]
    pub global: Global,

    /// Configuration for logging to the console.
    #[config(as_struct, as_type = "self::console::PartialConsole")]
    pub console: Console,

    /// Configuration for file logging.
    #[config(as_struct, as_type = "self::file::PartialFile")]
    pub file: File,
}

impl Logging {
    pub(crate) fn from_partial(partial: PartialLogging) -> Result<Self, LoggingLoadError> {
        let global = Global::from_partial(partial.global)?;
        let console = Console::from_partial(partial.console, &global)?;
        let file = File::from_partial(partial.file, &global)?;

        Ok(Self {
            global,
            console,
            file,
        })
    }
}

#[derive(Debug, Error)]
#[error("Could not load logging configuration")]
pub struct LoggingLoadError;

impl PartialLogging {
    pub fn from_env() -> Result<Self, LoggingLoadError> {
        let global = self::global::PartialGlobal::from_env()?;
        let console = self::console::PartialConsole::from_env()?;
        let file = self::file::PartialFile::from_env()?;

        Ok(Self {
            global,
            console,
            file,
        })
    }
}
