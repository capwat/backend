use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::Result;
use capwat_macros::ConfigParts;
use capwat_utils::{env, ProtectedString};
use doku::Document;
use serde::Deserialize;
use std::num::NonZeroU32;

use super::DBLoadError;

#[derive(Debug, Document, ConfigParts)]
#[config(attr(derive(Debug, Deserialize)))]
#[config(attr(serde(rename_all = "kebab-case")))]
pub struct DatabasePool {
    /// **Environment variables**:
    /// - `CAPWAT_DB_PRIMARY_MIN_CONNECTIONS` (for primary database)
    /// - `CAPWAT_DB_REPLICA_MIN_CONNECTIONS` (for replica database)
    ///
    /// Minimum amount of connections for Capwat to maintain
    /// it at all times.
    ///
    /// The minimum connections should not exceed to the maximum
    /// amount of comments (you may refer to max_connections, if you're
    /// unsure about its default value). However, the set value will be
    /// capped to `max_connections`.
    ///
    /// The default value is `0`, if not set.
    pub min_connections: u32,

    /// **Environment variables**:
    /// - `CAPWAT_DB_PRIMARY_MAX_CONNECTIONS` (for primary database)
    /// - `CAPWAT_DB_REPLICA_MAX_CONNECTIONS` (for replica database)
    ///
    /// Maximum amount of connections for Capwat to maintain
    /// it most of the time.
    ///
    /// The default is `10` connections, if not set.
    #[config(as_type = "Option<NonZeroU32>")]
    pub max_connections: u32,

    /// **Environment variables**:
    /// - `CAPWAT_DB_PRIMARY_READONLY_MODE` (for primary database)
    /// - `CAPWAT_DB_REPLICA_READONLY_MODE` (for replica database)
    ///
    /// Whether this database pool must be emulated as a
    /// read-only database.
    ///
    /// The default value is `false`, if not set.
    #[config(attr(serde(alias = "readonly")))]
    pub readonly_mode: bool,

    /// **Environment variables**:
    /// - `CAPWAT_DB_PRIMARY_URL` (for primary database)
    /// - `CAPWAT_DB_REPLICA_URL` (for replica database)
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
        let min_connections = partial.min_connections.unwrap_or(0);
        let max_connections = partial.max_connections.map(|v| v.get()).unwrap_or(10);

        let readonly_mode = partial.readonly_mode.unwrap_or(false);
        let url = partial.url.ok_or_else(|| capwat_error::Error::unknown(DBLoadError)).attach_printable_lazy(|| format!("`database.{db_type}.url` (in config file) or `CAPWAT_DB_{}_URL` (in environment variable) is required to connect to a {db_type} database", db_type.to_uppercase()))?;

        Ok(Self {
            min_connections,
            max_connections,
            readonly_mode,
            url,
        })
    }
}

impl PartialDatabasePool {
    pub(crate) fn from_env(db_type: &'static str) -> Result<Self, DBLoadError> {
        let min_connections =
            env::var_opt_parsed::<u32>(&format!("CAPWAT_DB_{db_type}_MIN_CONNECTIONS"))
                .change_context(DBLoadError)?;

        let max_connections =
            env::var_opt_parsed::<NonZeroU32>(&format!("CAPWAT_DB_{db_type}_CONNECTIONS"))
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
            max_connections,
            min_connections,
            readonly_mode,
            url: url.map(ProtectedString::new),
        })
    }

    pub fn has_defined(&self) -> bool {
        self.min_connections.is_some()
            || self.max_connections.is_some()
            || self.readonly_mode.is_some()
            || self.url.is_some()
    }
}
