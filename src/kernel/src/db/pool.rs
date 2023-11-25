use error_stack::{Report, Result, ResultExt};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use std::fmt::Debug;
use std::str::FromStr;
use thiserror::Error;

use super::{PoolConnection, SqlxErrorExt, Transaction};
use crate::config;

#[derive(Clone)]
pub struct Pool {
  inner: sqlx::PgPool,
}

#[derive(Debug, Error)]
pub enum PoolError {
  #[error("Invalid connection url")]
  InvalidUrl,
  #[error("Database is currently in read ome")]
  Readonly,
  #[error("Unhealthy database pool")]
  UnhealthyPool,
  #[error("received a pool error")]
  Internal,
}

impl Pool {
  #[tracing::instrument]
  pub async fn connect(
    global: &config::Database,
    pool: &config::DatabasePool,
  ) -> Result<Self, PoolError> {
    let mut pool_opts = PgPoolOptions::new()
      .acquire_timeout(global.timeout())
      .max_connections(pool.size());

    if let Some(min_idle) = pool.min_idle() {
      pool_opts = pool_opts.min_connections(min_idle);
    }

    let connect_opts = PgConnectOptions::from_str(pool.connection_url())
      .change_context(PoolError::Internal)?
      .ssl_mode(if global.enforces_tls() {
        PgSslMode::Prefer
      } else {
        PgSslMode::Allow
      });

    let pool = Self { inner: pool_opts.connect_lazy_with(connect_opts) };
    match pool.wait_until_healthy().await {
      Ok(..) => {},
      Err(err) if err.is_unhealthy() => {},
      Err(err) => return Err(err),
    }

    Ok(pool)
  }
}

impl Pool {
  #[inline]
  #[must_use]
  pub fn connections(&self) -> u32 {
    self.inner.size()
  }

  #[inline]
  #[must_use]
  pub fn is_healthy(&self) -> bool {
    self.inner.size() > 0
  }

  #[tracing::instrument(name = "db.transaction")]
  pub async fn begin(&self) -> Result<Transaction<'_>, PoolError> {
    if let Some(inner) = self.inner.try_begin().await.into_db_error()? {
      Ok(inner)
    } else if !self.is_healthy() {
      Err(PoolError::UnhealthyPool.into())
    } else {
      let result = self.inner.begin().await;
      result.map_err(|e| Report::new(e).change_context(PoolError::Internal))
    }
  }

  /// It attempts to get an active database connection.
  #[tracing::instrument(name = "db.connect")]
  pub async fn get(&self) -> Result<PoolConnection, PoolError> {
    if let Some(inner) = self.inner.try_acquire() {
      Ok(inner)
    } else if !self.is_healthy() {
      Err(PoolError::UnhealthyPool.into())
    } else {
      let result = self.inner.acquire().await;
      result.map_err(|e| Report::new(e).change_context(PoolError::Internal))
    }
  }

  #[tracing::instrument]
  pub async fn wait_until_healthy(&self) -> Result<(), PoolError> {
    match self.inner.acquire().await {
      Ok(..) => Ok(()),
      Err(e) if e.as_database_error().is_none() => {
        Err(Report::new(PoolError::UnhealthyPool))
      },
      Err(err) => Err(Report::new(err).change_context(PoolError::Internal)),
    }
  }
}

impl Debug for Pool {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.inner.fmt(f)
  }
}

pub(super) trait ErrorExt2 {
  fn is_unhealthy(&self) -> bool;
  fn is_readonly(&self) -> bool;
}

impl ErrorExt2 for error_stack::Report<PoolError> {
  fn is_unhealthy(&self) -> bool {
    self
      .downcast_ref::<PoolError>()
      .map(|v| matches!(v, PoolError::UnhealthyPool))
      .unwrap_or_default()
  }

  fn is_readonly(&self) -> bool {
    self
      .downcast_ref::<PoolError>()
      .map(|v| matches!(v, PoolError::Readonly))
      .unwrap_or_default()
  }
}
