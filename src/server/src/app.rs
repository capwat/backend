use axum::extract::{FromRequestParts, State};
use capwat_error::Result;
use capwat_postgres::error::{AcquireError, BeginTransactError};
use capwat_postgres::pool::PgConnection;
use capwat_postgres::transaction::Transaction;
use capwat_postgres::PgPool;
use capwat_vfs::Vfs;

use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use tracing::warn;

#[derive(Clone, FromRequestParts)]
#[from_request(via(State))]
#[must_use]
pub struct App(Arc<AppInner>);

impl App {
    /// Creates a new [`App`] from a given [configuration](capwat_config::Server).
    pub fn new(config: capwat_config::Server, vfs: Vfs) -> Self {
        let primary_db = PgPool::build(&config.database, &config.database.primary);
        let replica_db = config
            .database
            .replica
            .as_ref()
            .map(|replica| PgPool::build(&config.database, replica));

        Self(Arc::new(AppInner {
            config: Arc::new(config),
            primary_db,
            replica_db,
            vfs,
        }))
    }

    /// Creates a new [`App`] for testing purposes.
    #[cfg(test)]
    pub async fn new_for_tests(vfs: Vfs) -> Self {
        let primary_db = PgPool::build_for_tests().await;

        Self(Arc::new(AppInner {
            config: Arc::new(capwat_config::Server::for_tests()),
            primary_db,
            replica_db: None,
            vfs,
        }))
    }
}

impl App {
    /// Obtains a read/write database connection from the primary database pool.
    #[tracing::instrument(skip_all, name = "app.db_write")]
    pub async fn db_write(&self) -> Result<Transaction<'_>, BeginTransactError> {
        self.primary_db.begin_default().await
    }

    /// Obtains a readonly database connection from the replica
    /// pool or primary pool whichever is possible to obtain.
    ///
    /// The replica pool will be the first to obtain, if not,
    /// then the primary pool will be obtained instead.
    #[tracing::instrument(skip_all, name = "app.db_read")]
    pub async fn db_read(&self) -> Result<PgConnection<'_>, AcquireError> {
        let Some(replica_pool) = self.replica_db.as_ref() else {
            return self.primary_db.acquire().await;
        };

        match replica_pool.acquire().await {
            Ok(connection) => Ok(connection),
            Err(error) => {
                warn!(%error, "Replica database is not available, falling back to primary");
                self.primary_db.acquire().await
            }
        }
    }

    /// Obtains a readonly database connection from the primary pool.
    ///
    /// If the primary pool is not available, the replica pool will
    /// be used instead to obtain the connection.
    #[tracing::instrument(skip_all, name = "app.db_read_prefer_primary")]
    pub async fn db_read_prefer_primary(&self) -> Result<PgConnection<'_>, AcquireError> {
        let Some(replica_pool) = self.replica_db.as_ref() else {
            return self.primary_db.acquire().await;
        };

        match self.primary_db.acquire().await {
            Ok(connection) => Ok(connection),
            Err(error) => {
                warn!(%error, "Primary database is not available, falling back to replica");
                replica_pool.acquire().await
            }
        }
    }
}

impl Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("config", &self.config)
            .field("primary_db", &self.primary_db)
            .field("replica_db", &self.replica_db)
            .field("vfs", &self.vfs)
            .finish()
    }
}

impl Deref for App {
    type Target = AppInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Inner type of [`App`] object.
pub struct AppInner {
    pub config: Arc<capwat_config::Server>,
    pub primary_db: PgPool,
    pub replica_db: Option<PgPool>,
    pub vfs: Vfs,
}
