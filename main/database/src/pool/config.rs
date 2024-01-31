use capwat_diesel::PoolConfig;
use serde::Deserialize;
use std::num::NonZeroU64;
use std::time::Duration;

/// Global database configuration.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfig {
    /// Writable primary database.
    // #[validate(nested)]
    pub(crate) primary: PoolConfig,
    /// A read-only replica database used for accessing the data
    /// without interacting with the main database.
    // #[validate(nested, optional)]
    pub(crate) replica: Option<PoolConfig>,
    /// Forces all database connections are encrypted with TLS
    /// (if possible).
    #[serde(default = "default_enforce_tls")]
    pub(crate) enforce_tls: bool,
    /// How long this server can wait until its time limit where the
    /// database connection takes a while to acknowledge or
    /// successfully established.
    #[serde(default = "default_pool_timeout_secs")]
    pub(crate) timeout_secs: NonZeroU64,
}

impl DatabaseConfig {
    /// Gets the [`DbPoolConfig`] of a writable primary database.
    pub const fn primary(&self) -> &PoolConfig {
        &self.primary
    }

    /// Gets the [`DbPoolConfig`] of a readonly replica database.
    pub const fn replica(&self) -> Option<&PoolConfig> {
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

const fn default_pool_timeout_secs() -> NonZeroU64 {
    const DEFAULT_POOL_TIMEOUT_SECS: u64 = 5;
    match NonZeroU64::new(DEFAULT_POOL_TIMEOUT_SECS) {
        Some(n) => n,
        None => panic!("DEFAULT_POOL_TIMEOUT_SECS is accidentally set to 0"),
    }
}

const fn default_enforce_tls() -> bool {
    true
}
