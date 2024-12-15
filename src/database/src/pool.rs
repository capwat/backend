use capwat_error::{ext::ResultExt, ApiErrorCategory, Result};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::{fmt::Debug, time::Duration};
use thiserror::Error;

// Re-exports of sqlx's postgres stuff
pub use sqlx::PgConnection;

pub type PgTransaction<'q> = sqlx::Transaction<'q, sqlx::Postgres>;
pub type PgPooledConnection = sqlx::pool::PoolConnection<sqlx::Postgres>;

use crate::error::{AcquireError, BeginTransactionError};

/// An asynchronous pool of database connections.
///
/// This object is a pointer of [`sqlx::PgPool`] to retain
/// custom implementations with the pool code before the migration
/// from [`diesel`] to [`sqlx`] as our PostgreSQL database driver.
///
/// [`diesel`]: https://diesel.rs
#[derive(Clone)]
pub struct PgPool(sqlx::PgPool);

#[derive(Debug, Error)]
#[error("Could not build PgPool")]
pub struct BuildPoolError;

// We don't need to implement build_for_tests function anymore because we're going
// to replace with #[capwat_macros::test] for our main crates (server and worker).
impl PgPool {
    #[tracing::instrument(skip_all, name = "db.build_pool")]
    pub fn build(
        global: &capwat_config::DatabasePools,
        pool: &capwat_config::DatabasePool,
    ) -> Result<Self, BuildPoolError> {
        use sqlx::ConnectOptions;

        let stmt_timeout = global.statement_timeout;
        let connect_options = PgConnectOptions::from_url(&pool.url.expose())
            .change_context(BuildPoolError)
            .attach_printable("failed to parse PostgreSQL connection URL")?;

        let readonly_mode = pool.readonly_mode;
        let pool = PgPoolOptions::new()
            .idle_timeout(global.idle_timeout)
            .acquire_timeout(global.connection_timeout)
            .max_connections(pool.max_connections)
            .min_connections(pool.min_connections)
            .test_before_acquire(true)
            .after_connect(move |conn, _metadata| {
                Box::pin(async move {
                    sqlx::query(r"SET application_name = 'capwat'")
                        .execute(&mut *conn)
                        .await?;

                    let timeout = stmt_timeout.as_millis();
                    sqlx::query(&format!("SET statement_timeout = {timeout}"))
                        .execute(&mut *conn)
                        .await?;

                    if readonly_mode {
                        sqlx::query(r"SET default_transaction_read_only = 't'")
                            .execute(conn)
                            .await?;
                    }

                    Ok(())
                })
            })
            .connect_lazy_with(connect_options);

        Ok(Self(pool))
    }

    /// Attempts to acquire a connection from the pool.
    #[tracing::instrument(skip_all, name = "db.acquire")]
    pub async fn acquire(&self) -> Result<PgPooledConnection, AcquireError> {
        use sqlx::Error as SqlxError;
        match self.0.acquire().await {
            Ok(conn) => Ok(conn),
            result @ Err(
                SqlxError::PoolTimedOut | SqlxError::PoolClosed | SqlxError::WorkerCrashed,
            ) => match result {
                Err(error) => Err(error)
                    .change_context(AcquireError::Unhealthy)
                    .category(ApiErrorCategory::Outage),
                _ => unreachable!(),
            },
            Err(error) => Err(error).change_context(AcquireError::General),
        }
    }

    /// Attempts to perform a database transaction
    #[tracing::instrument(skip_all, name = "db.begin")]
    pub async fn begin(&self) -> Result<PgTransaction<'_>, BeginTransactionError> {
        self.0.begin().await.change_context(BeginTransactionError)
    }

    /// Checks whether the database pool connection is healthy.
    #[tracing::instrument(skip_all, name = "db.check_health")]
    pub async fn check_health(&self, timeout: Option<Duration>) -> Result<bool> {
        let tester = async {
            let mut conn = match self.acquire().await {
                Ok(conn) => conn,
                Err(error) => match error.current_context() {
                    AcquireError::Unhealthy => return Ok(false),
                    _ => return Err(error.into()),
                },
            };

            // TODO: Check if it is safe to cancel this query by canceling the query future.
            sqlx::query("SELECT 1").execute(&mut *conn).await?;
            Ok::<bool, capwat_error::Error>(true)
        };

        let timeout = timeout.unwrap_or(Duration::from_secs(5));
        match tokio::time::timeout(timeout, tester).await {
            Ok(result) => result,
            Err(..) => Ok(false),
        }
    }

    #[must_use]
    pub fn connections(&self) -> u32 {
        self.0.size()
    }

    #[must_use]
    pub fn idle_connections(&self) -> usize {
        self.0.num_idle()
    }
}

impl Debug for PgPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl From<sqlx::PgPool> for PgPool {
    fn from(value: sqlx::PgPool) -> Self {
        Self(value)
    }
}
