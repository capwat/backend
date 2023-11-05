use sensitive::Sensitive;
use serde::Deserialize;
use std::num::{NonZeroU32, NonZeroU64};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Database {
    /// Writable primary database.
    #[validate(nested)]
    pub primary: DbPoolConfig,
    /// A read-only replica database used for accessing the data
    /// without interacting with the main database.
    #[validate(nested, optional)]
    pub replica: Option<DbPoolConfig>,
    /// Forces all database connections are encrypted with TLS
    /// (if possible).
    ///
    /// **Environment variables**:
    /// - `WHIM_DB_ENFORCE_TLS`
    #[serde(default = "DbPoolConfig::default_enforce_tls")]
    pub enforce_tls: bool,
    /// How long this server can wait until its time limit where the
    /// database connection takes a while to acknowledge or
    /// successfully established.
    ///
    /// **Environment variables**:
    /// - `WHIM_DB_TIMEOUT_SECS`
    #[serde(default = "DbPoolConfig::default_pool_timeout_secs")]
    pub timeout_secs: NonZeroU64,
}

/// Configuration for connecting to any Postgres database
#[derive(Debug, Deserialize, Validate)]
pub struct DbPoolConfig {
    /// Database pool must be in read-only mode.
    ///
    /// **Environment variables**:
    /// - `WHIM_DB_PRIMARY_READONLY`
    /// - `WHIM_DB_REPLICA_READONLY`
    #[serde(default)]
    pub readonly: bool,
    /// Minimum idle database connections just to avoid wasting
    /// hardware resources from the database server.
    ///
    /// **Environment variables**:
    /// - `WHIM_DB_PRIMARY_MIN_IDLE`
    /// - `WHIM_DB_REPLICA_MIN_IDLE`
    pub min_idle: Option<NonZeroU32>,
    /// Maximum amount of pool size that database can handle
    ///
    /// **Environment variables**:
    /// - `WHIM_DB_PRIMARY_POOL_SIZE`
    /// - `WHIM_DB_REPLICA_POOL_SIZE`
    #[serde(default = "DbPoolConfig::default_pool_size")]
    pub pool_size: NonZeroU32,
    /// Connection URL connecting to the Postgres database.
    ///
    /// **Environment variables**:
    /// - `WHIM_DB_PRIMARY_URL` or `DATABASE_URL`
    /// - `WHIM_DB_REPLICA_URL`
    #[validate(
        with = "validator::extras::validate_url",
        error = "Invalid Postgres connection URL"
    )]
    pub url: Sensitive<String>,
}

impl DbPoolConfig {
    const DEFAULT_POOL_SIZE: u32 = 5;
    const DEFAULT_POOL_TIMEOUT_SECS: u64 = 5;

    // Required by serde
    const fn default_pool_size() -> NonZeroU32 {
        match NonZeroU32::new(Self::DEFAULT_POOL_SIZE) {
            Some(n) => n,
            None => panic!("DEFAULT_POOL_SIZE is accidentally set to 0"),
        }
    }

    const fn default_pool_timeout_secs() -> NonZeroU64 {
        match NonZeroU64::new(Self::DEFAULT_POOL_TIMEOUT_SECS) {
            Some(n) => n,
            None => panic!("DEFAULT_POOL_TIMEOUT_SECS is accidentally set to 0"),
        }
    }

    const fn default_enforce_tls() -> bool {
        true
    }
}
