use error_stack::{Report, ResultExt};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use std::str::FromStr;
use whim_core::config;

use crate::{
  error::{ErrorExt, ErrorExt2},
  Error, PoolConnection, Result, Transaction,
};

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
  #[inline(always)]
  pub fn connections(&self) -> u32 {
    self.pool.size()
  }

  #[inline(always)]
  pub fn is_healthy(&self) -> bool {
    self.connections() > 0
  }

  #[doc(hidden)]
  #[tracing::instrument(name = "db.transaction", skip(self))]
  pub async fn begin(&self) -> Result<Transaction> {
    if let Some(inner) = self.pool.try_begin().await.into_db_error()? {
      Ok(inner)
    } else if !self.is_healthy() {
      Err(Error::UnhealthyPool.into())
    } else {
      let result = self.pool.begin().await;
      result.map_err(|e| Report::new(Error::Internal(e)))
    }
  }

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

  #[tracing::instrument(skip(self))]
  pub async fn wait_until_healthy(&self) -> Result<()> {
    match self.pool.acquire().await {
      Ok(..) => Ok(()),
      Err(e) if !self.is_healthy() => Err(e).change_context(Error::UnhealthyPool),
      Err(err) => Err(Report::new(Error::Internal(err))),
    }
  }
}
