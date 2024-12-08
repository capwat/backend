use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::Result;
use capwat_macros::ConfigParts;
use capwat_utils::env;
use doku::Document;
use serde::Deserialize;
use serde_with::serde_as;
use std::time::Duration;
use thiserror::Error;

use self::pool::PartialDatabasePool;
use crate::vars;

pub(crate) mod pool;
pub use self::pool::DatabasePool;

#[derive(Debug, Document, ConfigParts)]
#[config(attr(serde_as))]
#[config(attr(derive(Debug, Deserialize)))]
#[config(attr(serde(rename_all = "kebab-case")))]
pub struct DatabasePools {
    /// Connection configuration for the primary database
    #[config(as_struct, as_type = "PartialDatabasePool")]
    pub primary: DatabasePool,

    /// Connection configuration for the replica database
    #[config(as_struct, as_type = "Option<PartialDatabasePool>")]
    pub replica: Option<DatabasePool>,

    /// **Environment variable**: `CAPWAT_DB_ENFORCE_TLS`
    ///
    /// Whether connecting to the database with TLS encrypted
    /// tunnel is enabled.
    pub enforce_tls: bool,

    /// **Environment variable**: `CAPWAT_DB_CONNECTION_TIMEOUT`
    ///
    /// Maximum amount of time to spend waiting for the database
    /// to successfully establish connection.
    ///
    /// Capwat will reject any database related transactions if it exceeds
    /// the maximum amount of time waiting to successfully connect to
    /// the database.
    ///
    /// The default is `15` seconds, if not set.
    #[config(attr(serde_as(as = "capwat_utils::serde_exts::AsHumanDuration")))]
    #[config(attr(serde(default)))]
    pub connection_timeout: Duration,

    /// **Environment variable**: `CAPWAT_DB_IDLE_TIMEOUT`
    ///
    /// Maximum idle duration for individual pooled connections.
    ///
    /// Any connection remains idle longer than the configured
    /// will be closed.
    ///
    /// The default is `10` minutes, if not set.
    #[config(attr(serde_as(as = "capwat_utils::serde_exts::AsHumanDuration")))]
    #[config(attr(serde(default)))]
    pub idle_timeout: Duration,

    /// **Environment variable**: `CAPWAT_DB_STATEMENT_TIMEOUT`
    ///
    /// Maximum period of time that every transaction in the database
    /// must be performed before it will cancel the transaction if it
    /// exceeds the threshold.
    ///
    /// The default is `15` seconds, if not set.
    #[config(attr(serde_as(as = "capwat_utils::serde_exts::AsHumanDuration")))]
    #[config(attr(serde(default)))]
    pub statement_timeout: Duration,
}

impl DatabasePools {
    pub(crate) fn from_partial(partial: PartialDatabasePools) -> Result<Self, DBLoadError> {
        let primary = DatabasePool::from_partial(partial.primary, "primary")
            .attach_printable("could not load primary database pool configuration")?;

        let replica = if let Some(replica) = partial.replica {
            let inner = DatabasePool::from_partial(replica, "replica")
                .attach_printable("could not load replica database pool configuration")?;

            Some(inner)
        } else {
            None
        };

        let enforce_tls = partial.enforce_tls.unwrap_or(false);
        let connection_timeout = partial
            .connection_timeout
            .unwrap_or(Duration::from_secs(15));

        let idle_timeout = partial
            .statement_timeout
            .unwrap_or(Duration::from_secs(60 * 10));

        let statement_timeout = partial.statement_timeout.unwrap_or(Duration::from_secs(15));

        Ok(Self {
            primary,
            replica,
            enforce_tls,
            connection_timeout,
            idle_timeout,
            statement_timeout,
        })
    }
}

#[derive(Debug, Error)]
#[error("Could not load database configuration")]
pub struct DBLoadError;

impl PartialDatabasePools {
    pub(crate) fn from_env() -> Result<Self, DBLoadError> {
        let primary = PartialDatabasePool::from_env("PRIMARY")
            .attach_printable("could not load primary database pool configuration")?;

        let replica = PartialDatabasePool::from_env("REPLICA")
            .attach_printable("could not load replica database pool configuration")?;

        let enforce_tls =
            env::var_opt_parsed::<bool>(vars::DB_ENFORCE_TLS).change_context(DBLoadError)?;

        let connection_timeout = env::var_opt_parsed_fn(vars::DB_CONNECTION_TIMEOUT, |value| {
            capwat_utils::time::parse_from_human_duration(value)
        })
        .change_context(DBLoadError)?;

        let idle_timeout = env::var_opt_parsed_fn(vars::DB_IDLE_TIMEOUT, |value| {
            capwat_utils::time::parse_from_human_duration(value)
        })
        .change_context(DBLoadError)?;

        let statement_timeout = env::var_opt_parsed_fn(vars::DB_STATEMENT_TIMEOUT, |value| {
            capwat_utils::time::parse_from_human_duration(value)
        })
        .change_context(DBLoadError)?;

        Ok(Self {
            primary,
            replica: if replica.has_defined() {
                Some(replica)
            } else {
                None
            },
            enforce_tls,
            connection_timeout,
            idle_timeout,
            statement_timeout,
        })
    }
}
