use error_stack::{Result, ResultExt};
use std::sync::Arc;
use thiserror::Error;
use whim_core::config;
use whim_database::error::ErrorExt2;

#[derive(Debug, Clone)]
pub struct App {
  pub config: Arc<config::Server>,
  pub primary_db: whim_database::Pool,
  pub replica_db: Option<whim_database::Pool>,
}

#[derive(Debug, Error)]
#[error("Failed to initialize App struct")]
pub struct Error;

impl App {
  #[tracing::instrument]
  pub async fn new(cfg: config::Server) -> Result<Self, Error> {
    let db_cfg = cfg.db();
    let primary_db = whim_database::Pool::new(&db_cfg, &db_cfg.primary())
      .await
      .change_context(Error)?;

    let replica_db = if let Some(replica) = db_cfg.replica().as_ref() {
      let pool = whim_database::Pool::new(&db_cfg, replica)
        .await
        .change_context(Error)?;

      Some(pool)
    } else {
      None
    };

    let app = Self {
      config: Arc::new(cfg),
      primary_db,
      replica_db,
    };

    Ok(app)
  }
}

impl App {
  #[tracing::instrument(skip_all)]
  pub async fn db_write(&self) -> Result<whim_database::PoolConnection, whim_database::Error> {
    Ok(self.primary_db.get().await?)
  }

  #[tracing::instrument(skip_all)]
  pub async fn db_read(&self) -> Result<whim_database::PoolConnection, whim_database::Error> {
    if let Some(replica) = self.replica_db.as_ref() {
      match replica.get().await {
        Ok(conn) => return Ok(conn),
        // fallback
        Err(err) if err.is_unhealthy() => {}
        Err(err) => return Err(err.into()),
      }
    }
    self.primary_db.get().await
  }

  #[tracing::instrument(skip_all)]
  pub async fn db_read_prefer_primary(
    &self,
  ) -> Result<whim_database::PoolConnection, whim_database::Error> {
    match (self.primary_db.get().await, self.replica_db.as_ref()) {
      (Ok(conn), ..) => Ok(conn),
      (Err(e), Some(readonly_replica)) if e.is_unhealthy() => Ok(readonly_replica.get().await?),
      (Err(e), ..) => Err(e.into()),
    }
  }
}
