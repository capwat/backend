use error_stack::{Result, ResultExt};
use std::sync::Arc;
use thiserror::Error;

use crate::{
    config,
    database::{self, ErrorExt2},
};

#[derive(Debug, Clone)]
pub struct App {
    pub config: Arc<config::Server>,
    pub primary_db: database::Pool,
    pub replica_db: Option<database::Pool>,
}

#[derive(Debug, Error)]
#[error("Failed to initialize App struct")]
pub struct AppError;

impl App {
    #[tracing::instrument]
    pub async fn new(cfg: config::Server) -> Result<Self, AppError> {
        let primary_db = database::Pool::new(&cfg.db, &cfg.db.primary)
            .await
            .change_context(AppError)?;

        let replica_db = if let Some(replica) = cfg.db.replica.as_ref() {
            Some(
                database::Pool::new(&cfg.db, replica)
                    .await
                    .change_context(AppError)?,
            )
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
    pub async fn db_write(&self) -> Result<database::PoolConnection, database::Error> {
        Ok(self.primary_db.get().await?)
    }

    #[tracing::instrument(skip_all)]
    pub async fn db_read(&self) -> Result<database::PoolConnection, database::Error> {
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
    ) -> Result<database::PoolConnection, database::Error> {
        match (self.primary_db.get().await, self.replica_db.as_ref()) {
            (Ok(conn), ..) => Ok(conn),
            (Err(e), Some(readonly_replica)) if e.is_unhealthy() => {
                Ok(readonly_replica.get().await?)
            }
            (Err(e), ..) => Err(e.into()),
        }
    }
}
