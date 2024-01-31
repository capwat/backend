use capwat_kernel::util::Sensitive;
use serde::Deserialize;
use std::num::{NonZeroU32, NonZeroUsize};

/// Configuration for connecting to any Postgres database
#[derive(Debug, Deserialize)]
pub struct PoolConfig {
    /// Database pool must be in read-only mode.
    #[serde(default)]
    pub(crate) readonly: bool,
    /// Minimum idle database connections just to avoid wasting
    /// hardware resources from the database server.
    pub(crate) min_idle: Option<NonZeroU32>,
    /// Maximum amount of pool size that database can handle
    #[serde(default = "PoolConfig::default_pool_size")]
    pub(crate) pool_size: NonZeroUsize,
    /// Connection URL connecting to the Postgres database.
    // #[validate(
    //     with = "DbPoolConfig::validate_pg_url",
    //     error = "Invalid Postgres connection URL"
    // )]
    pub(crate) url: Sensitive<String>,
}

impl PoolConfig {
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
    pub const fn pool_size(&self) -> usize {
        self.pool_size.get()
    }

    /// Gets the connection URL to connect to the database
    pub const fn connection_url(&self) -> &Sensitive<String> {
        &self.url
    }
}

impl PoolConfig {
    const DEFAULT_POOL_SIZE: usize = 5;

    // Required by serde
    const fn default_pool_size() -> NonZeroUsize {
        match NonZeroUsize::new(Self::DEFAULT_POOL_SIZE) {
            Some(n) => n,
            None => panic!("DEFAULT_POOL_SIZE is accidentally set to 0"),
        }
    }

    // fn validate_pg_url(url: &str) -> bool {
    //     let mut accepted = false;
    //     if let Ok(url) = url::Url::parse(url) {
    //         accepted = url.as_str().starts_with("postgres://")
    //             && url.scheme() == "postgres";
    //     }
    //     accepted
    // }
}
