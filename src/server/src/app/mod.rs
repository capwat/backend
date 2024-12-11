use axum::extract::{FromRequestParts, State};
use capwat_db::error::{AcquireError, BeginTransactError};
use capwat_db::pool::PgConnection;
use capwat_db::transaction::Transaction;
use capwat_db::PgPool;
use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::Result;
use capwat_vfs::Vfs;
use std::sync::Arc;
use thiserror::Error;
use tracing::{trace, warn};

use self::private::AppInner;

#[derive(Clone, FromRequestParts)]
#[from_request(via(State))]
#[must_use]
pub struct App(Arc<AppInner>);

#[derive(Debug, Error)]
#[error("Could not initialize server application")]
pub struct AppError;

impl App {
    pub fn new(config: capwat_config::Server, vfs: Vfs) -> Result<Self, AppError> {
        let primary_db = PgPool::build(&config.database, &config.database.primary);
        let replica_db = config
            .database
            .replica
            .as_ref()
            .map(|replica| PgPool::build(&config.database, replica));

        let (jwt_encode, jwt_decode) = Self::setup_jwt_keys(&config, &vfs)
            .change_context(AppError)
            .attach_printable("could not setup JWT authentication")?;

        let inner = Arc::new(AppInner {
            config: Arc::new(config),
            vfs,

            primary_db,
            replica_db,

            jwt_encode,
            jwt_decode,
        });

        Ok(Self(inner))
    }

    /// Creates a new [`App`] for testing purposes.
    pub async fn new_for_tests(vfs: Vfs) -> Self {
        let primary_db = PgPool::build_for_tests(&capwat_model::DB_MIGRATIONS).await;
        let config = capwat_config::Server::for_tests();
        let (jwt_encode, jwt_decode) = Self::setup_jwt_keys(&config, &vfs)
            .change_context(AppError)
            .attach_printable("could not setup JWT authentication")
            .unwrap();

        Self(Arc::new(AppInner {
            config: Arc::new(config),
            primary_db,
            replica_db: None,
            vfs,

            jwt_encode,
            jwt_decode,
        }))
    }
}

impl App {
    /// Obtains a read/write database connection from the primary database pool.
    #[tracing::instrument(skip_all, name = "app.db_write")]
    pub async fn db_write(&self) -> Result<Transaction<'_>, BeginTransactError> {
        trace!("obtaining primary db connection...");
        self.primary_db.begin_default().await
    }

    /// Obtains a readonly database connection from the replica
    /// pool or primary pool whichever is possible to obtain.
    ///
    /// The replica pool will be the first to obtain, if not,
    /// then the primary pool will be obtained instead.
    #[tracing::instrument(skip_all, name = "app.db_read")]
    pub async fn db_read(&self) -> Result<PgConnection<'_>, AcquireError> {
        trace!("obtaining replica db connection...");

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
        trace!("obtaining primary db connection...");

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

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("config", &self.config)
            .field("primary_db", &self.primary_db)
            .field("replica_db", &self.replica_db)
            .field("vfs", &self.vfs)
            .finish()
    }
}

impl std::ops::Deref for App {
    type Target = AppInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub mod auth;

mod private;
mod validators;
