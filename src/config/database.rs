use serde::Deserialize;
use std::{
  num::{NonZeroU32, NonZeroU64},
  time::Duration,
};
use validator::Validate;

use crate::util::Sensitive;

/// Global database configuration.
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Database {
  /// Writable primary database.
  #[validate(nested)]
  pub(crate) primary: DbPoolConfig,
  /// A read-only replica database used for accessing the data
  /// without interacting with the main database.
  #[validate(nested, optional)]
  pub(crate) replica: Option<DbPoolConfig>,
  /// Forces all database connections are encrypted with TLS
  /// (if possible).
  ///
  /// **Environment variables**:
  /// - `WHIM_DB_ENFORCE_TLS`
  #[serde(default = "DbPoolConfig::default_enforce_tls")]
  pub(crate) enforce_tls: bool,
  /// How long this server can wait until its time limit where the
  /// database connection takes a while to acknowledge or
  /// successfully established.
  ///
  /// **Environment variables**:
  /// - `WHIM_DB_TIMEOUT_SECS`
  #[serde(default = "DbPoolConfig::default_pool_timeout_secs")]
  pub(crate) timeout_secs: NonZeroU64,
}

impl Database {
  /// Gets the [`DbPoolConfig`] of a writable primary database.
  pub const fn primary(&self) -> &DbPoolConfig {
    &self.primary
  }

  /// Gets the [`DbPoolConfig`] of a readonly replica database.
  pub const fn replica(&self) -> Option<&DbPoolConfig> {
    self.replica.as_ref()
  }

  /// Whether or not it forces TLS/all database connections
  /// to be encrypted or connect to the database with TLS (SSL but newer).
  pub const fn enforces_tls(&self) -> bool {
    self.enforce_tls
  }

  /// How long this server can wait until its time limit where the
  /// database connection takes a while to acknowledge or
  /// successfully established.
  pub const fn timeout(&self) -> Duration {
    Duration::from_secs(self.timeout_secs.get())
  }

  /// How long this server can wait until its time limit where the
  /// database connection takes a while to acknowledge or
  /// successfully established.
  pub const fn timeout_secs(&self) -> u64 {
    self.timeout_secs.get()
  }
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
  pub(crate) readonly: bool,
  /// Minimum idle database connections just to avoid wasting
  /// hardware resources from the database server.
  ///
  /// **Environment variables**:
  /// - `WHIM_DB_PRIMARY_MIN_IDLE`
  /// - `WHIM_DB_REPLICA_MIN_IDLE`
  pub(crate) min_idle: Option<NonZeroU32>,
  /// Maximum amount of pool size that database can handle
  ///
  /// **Environment variables**:
  /// - `WHIM_DB_PRIMARY_POOL_SIZE`
  /// - `WHIM_DB_REPLICA_POOL_SIZE`
  #[serde(default = "DbPoolConfig::default_pool_size")]
  pub(crate) pool_size: NonZeroU32,
  /// Connection URL connecting to the Postgres database.
  ///
  /// **Environment variables**:
  /// - `WHIM_DB_PRIMARY_URL` or `DATABASE_URL`
  /// - `WHIM_DB_REPLICA_URL`
  #[validate(
    with = "DbPoolConfig::validate_pg_url",
    error = "Invalid Postgres connection URL"
  )]
  pub(crate) url: Sensitive<String>,
}

impl DbPoolConfig {
  /// Whether database pool must not write anything
  pub const fn readonly(&self) -> bool {
    self.readonly
  }

  /// Gets the minimum idle connections for a database pool
  pub const fn min_idle(&self) -> Option<u32> {
    match self.min_idle {
      Some(v) => Some(v.get()),
      None => None,
    }
  }

  /// Gets the maximum connections for a database pool
  pub const fn pool_size(&self) -> u32 {
    self.pool_size.get()
  }

  /// Gets the connection URL to connect to the database
  pub const fn connection_url(&self) -> &Sensitive<String> {
    &self.url
  }
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

  fn validate_pg_url(url: &str) -> bool {
    let mut accepted = false;
    if let Ok(url) = url::Url::parse(url) {
      accepted = url.as_str().starts_with("postgres://") && url.scheme() == "postgres";
    }
    accepted
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::hint::black_box;

  #[test]
  fn test_consts_not_crashing() {
    black_box(DbPoolConfig::default_pool_size().get());
    black_box(DbPoolConfig::default_pool_timeout_secs().get());
  }

  #[test]
  fn test_validate_pg_url() {
    assert!(DbPoolConfig::validate_pg_url("postgres://hello.world"));
    assert!(!DbPoolConfig::validate_pg_url("hello.world"));
  }
}
