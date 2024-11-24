use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::Result;
use capwat_macros::ConfigParts;
use capwat_utils::{env, ProtectedString};
use doku::Document;
use serde::Deserialize;
use std::num::NonZeroUsize;

use super::DBLoadError;

#[derive(Debug, Document, ConfigParts)]
#[config(attr(derive(Debug, Deserialize)))]
pub struct DatabasePool {
    /// Maximum amount of connections which this pool can handle.
    ///
    /// The default value is `1`, if not set.
    #[config(as_type = "Option<NonZeroUsize>")]
    pub connections: usize,

    /// Whether this database pool must be emulated as a
    /// read-only database.
    ///
    /// The default value is `false`, if not set.
    pub readonly_mode: bool,

    #[doku(
        as = "String",
        example = "postgres://user:password@localhost:5432/capwat"
    )]
    pub url: ProtectedString,
}

impl DatabasePool {
    pub(crate) fn from_partial(
        partial: PartialDatabasePool,
        db_type: &'static str,
    ) -> Result<Self, DBLoadError> {
        let connections = partial.connections.map(|v| v.get()).unwrap_or(1);
        let readonly_mode = partial.readonly_mode.unwrap_or(false);
        let url = partial.url.ok_or_else(|| capwat_error::Error::unknown(DBLoadError)).attach_printable_lazy(|| format!("`database.{db_type}.url` (in config file) or `CAPWAT_DB_{}_URL` (in environment variable) is required to connect to a {db_type} database", db_type.to_uppercase()))?;

        Ok(Self {
            connections,
            readonly_mode,
            url,
        })
    }
}

impl PartialDatabasePool {
    pub(crate) fn from_env(db_type: &'static str) -> Result<Self, DBLoadError> {
        let connections =
            env::var_opt_parsed::<NonZeroUsize>(&format!("CAPWAT_DB_{db_type}_CONNECTIONS"))
                .change_context(DBLoadError)?;

        let readonly_mode =
            env::var_opt_parsed::<bool>(&format!("CAPWAT_DB_{db_type}_READONLY_MODE"))
                .change_context(DBLoadError)?;

        let url = env::var_opt(&format!("CAPWAT_DB_{db_type}_URL"))
            .change_context(DBLoadError)
            .attach_printable_lazy(|| {
                format!(
                    "`CAPWAT_DB_{db_type}_URL` is required to setup to connect to a {} database",
                    db_type.to_lowercase()
                )
            })?;

        Ok(Self {
            connections,
            readonly_mode,
            url: url.map(|v| ProtectedString::new(v)),
        })
    }

    pub fn has_defined(&self) -> bool {
        self.connections.is_some() || self.readonly_mode.is_some() || self.url.is_some()
    }
}
