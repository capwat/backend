pub(crate) mod ext;
mod pool;

pub use self::pool::{Pool, PoolError};

pub type Transaction<'a> = sqlx::Transaction<'a, sqlx::Postgres>;
pub type PoolConnection = sqlx::pool::PoolConnection<sqlx::Postgres>;
pub type Connection = sqlx::PgConnection;

use error_stack::Result;

use self::ext::DbErrorExt2;
use crate::config;

#[derive(Debug, Clone)]
pub struct Database {
  primary: Pool,
  replica: Option<Pool>,
}

impl Database {
  #[tracing::instrument]
  pub async fn connect(cfg: &config::Database) -> Result<Self, PoolError> {
    let primary = Pool::connect(cfg, cfg.primary()).await?;
    let replica = if let Some(pool_cfg) = cfg.replica() {
      Some(Pool::connect(cfg, pool_cfg).await?)
    } else {
      None
    };

    Ok(Self { primary, replica })
  }

  #[must_use]
  pub fn primary(&self) -> &Pool {
    &self.primary
  }

  #[must_use]
  pub fn replica(&self) -> Option<&Pool> {
    self.replica.as_ref()
  }
}

impl Database {
  #[tracing::instrument]
  pub async fn write(&self) -> Result<Transaction<'_>, PoolError> {
    self.primary.begin().await
  }

  #[tracing::instrument]
  pub async fn read(&self) -> Result<PoolConnection, PoolError> {
    if let Some(replica) = self.replica.as_ref() {
      match replica.get().await {
        Ok(conn) => return Ok(conn),
        // fallback
        Err(err) if err.is_unhealthy() => {},
        Err(err) => return Err(err),
      }
    }
    self.primary.get().await
  }

  #[tracing::instrument]
  pub async fn read_prefer_primary(&self) -> Result<PoolConnection, PoolError> {
    match (self.primary.get().await, self.replica.as_ref()) {
      (Ok(conn), ..) => Ok(conn),
      (Err(e), Some(replica)) if e.is_unhealthy() => Ok(replica.get().await?),
      (Err(e), ..) => Err(e),
    }
  }
}
