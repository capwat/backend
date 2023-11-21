use error_stack::{Report, ResultExt};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use std::str::FromStr;
use whim_core::config;

use crate::{
  error::{ErrorExt, ErrorExt2},
  Error, PoolConnection, Result, Transaction,
};

/// A Postgres database connection pool.
///
/// To establish a Postgres database pool, one must have a
/// [global database config](config::Database) and the [pool config](config::DbPoolConfig)
/// used to connect to the database.
///
/// [Global database config](config::Database) will be applied in common
/// configurations such as `timeout_secs`. Meanwhile, [pool config](config::DbPoolConfig)
/// will be applied specifically for database connection pool.
#[derive(Clone)]
pub struct Pool {
  pool: sqlx::PgPool,
}

impl Pool {
  /// Creates and tests a database from a database global
  /// and pool configuration.
  pub async fn new(
    global_cfg: &config::Database,
    pool_cfg: &config::DbPoolConfig,
    // time_to_obtain_connection_metric: Histogram,
  ) -> Result<Self> {
    let mut pool_opts = PgPoolOptions::new()
      .acquire_timeout(global_cfg.timeout())
      .max_connections(pool_cfg.pool_size());

    if let Some(min_idle) = pool_cfg.min_idle() {
      pool_opts = pool_opts.min_connections(min_idle);
    }

    let mut connect_opts =
      PgConnectOptions::from_str(&*pool_cfg.connection_url()).change_context(Error::InvalidUrl)?;

    if global_cfg.enforces_tls() {
      connect_opts = connect_opts.ssl_mode(PgSslMode::Prefer);
    }

    let pool = Self {
      pool: pool_opts.connect_lazy_with(connect_opts),
    };

    match pool.wait_until_healthy().await {
      Ok(..) => {}
      Err(err) if err.is_unhealthy() => {}
      Err(err) => return Err(err),
    }

    Ok(pool)
  }
}

impl std::fmt::Debug for Pool {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.pool.fmt(f)
  }
}

impl Pool {
  /// Gets the active connections of a database pool
  #[inline(always)]
  pub fn connections(&self) -> u32 {
    self.pool.size()
  }

  /// Checks if the database pool is healthy.
  ///
  /// This function uses `.connections()` method (a method
  /// used to get active connections in [Pool object](Pool)) and
  /// check if it is greater than `0``.
  #[inline(always)]
  pub fn is_healthy(&self) -> bool {
    self.connections() > 0
  }

  /// It attempts to start a database transaction and returns
  /// a connection with transaction is active until it is dropped.
  ///
  /// For more instructions on how to use the [`Transaction`](crate::Transaction) object,
  /// please refer to [sqlx's Transaction object documentation](sqlx::Transaction)
  #[doc(hidden)]
  #[tracing::instrument(name = "db.transaction", skip(self))]
  pub async fn begin(&self) -> Result<Transaction<'_>> {
    if let Some(inner) = self.pool.try_begin().await.into_db_error()? {
      Ok(inner)
    } else if !self.is_healthy() {
      Err(Error::UnhealthyPool.into())
    } else {
      let result = self.pool.begin().await;
      result.map_err(|e| Report::new(Error::Internal(e)))
    }
  }

  /// It attempts to get an active database connection.
  #[tracing::instrument(name = "db.connect", skip(self))]
  pub async fn get(&self) -> Result<PoolConnection> {
    if let Some(inner) = self.pool.try_acquire() {
      Ok(inner)
    } else if !self.is_healthy() {
      Err(Error::UnhealthyPool.into())
    } else {
      let result = self.pool.acquire().await;
      result.map_err(|e| Report::new(Error::Internal(e)))
    }
  }

  /// This function will try to wait for a database connection
  /// to be successfully established until there's a timeout
  /// (can be configured through [`config.timeout_secs`](config::Database)).
  ///
  /// If it fails to connect for a certain period of time, it
  /// will throw an error stating that there's something wrong
  /// when establishing a connection to the database.
  #[tracing::instrument(skip(self))]
  pub async fn wait_until_healthy(&self) -> Result<()> {
    match self.pool.acquire().await {
      Ok(..) => Ok(()),
      Err(e @ sqlx::Error::PoolTimedOut) => Err(e).change_context(Error::UnhealthyPool),
      Err(err) => Err(Report::new(Error::Internal(err))),
    }
  }
}
