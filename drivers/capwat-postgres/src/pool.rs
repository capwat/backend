use async_trait::async_trait;
use bb8::{CustomizeConnection, Pool};
use capwat_error::ext::ResultExt;
use capwat_error::Result;
use diesel_async::pooled_connection::PoolError;
use diesel_async::pooled_connection::{
    bb8::PooledConnection, AsyncDieselConnectionManager, ManagerConfig,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::MutexGuard;
use tracing::warn;

use crate::error::{AcquireError, BeginTransactError};
use crate::internal::AnyPool;
use crate::transaction::{Transaction, TransactionBuilder};

#[derive(Clone)]
pub struct PgPool(Arc<dyn AnyPool>);

impl PgPool {
    #[tracing::instrument(skip_all)]
    pub fn build(
        global: &capwat_config::DatabasePools,
        pool: &capwat_config::DatabasePool,
    ) -> Self {
        let mut config = ManagerConfig::default();
        if global.enforce_tls {
            config.custom_setup = Box::new(crate::internal::establish_connection_with_tls);
        }

        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
            pool.url.expose(),
            config,
        );

        let pool = Pool::builder()
            .connection_timeout(global.connection_timeout)
            .min_idle(Some(pool.min_connections))
            .max_size(pool.max_connections)
            .idle_timeout(Some(global.idle_timeout))
            .connection_customizer(Box::new(CustomDbConnector {
                readonly_mode: pool.readonly_mode,
                statement_timeout: global.statement_timeout,
            }))
            .build_unchecked(manager);

        Self(Arc::new(pool))
    }

    #[tracing::instrument(name = "db.build_for_tests")]
    pub async fn build_for_tests() -> Self {
        let pool = crate::test::TestPool::connect().await;
        Self(Arc::new(pool))
    }

    /// Attempts to acquire a connection from the pool.
    pub async fn acquire(&self) -> Result<PgConnection<'_>, AcquireError> {
        self.0.acquire().await
    }

    /// Attempts to perform a database transaction
    pub async fn begin(&self) -> Result<TransactionBuilder<'_>, BeginTransactError> {
        let conn = self.acquire().await.change_context(BeginTransactError)?;
        Ok(TransactionBuilder::new(conn, self.0.is_testing()))
    }

    /// Attempts to perform a database transaction without any configuration needed.
    pub async fn begin_default(&self) -> Result<Transaction<'_>, BeginTransactError> {
        let conn = self.acquire().await.change_context(BeginTransactError)?;
        TransactionBuilder::new(conn, self.0.is_testing())
            .build()
            .await
    }

    #[tracing::instrument(skip(self), name = "db.check_health")]
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
            diesel::sql_query("SELECT 1;").execute(&mut conn).await?;
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
        self.0.connections()
    }

    #[must_use]
    pub fn idle_connections(&self) -> u32 {
        self.0.idle_connections()
    }
}

impl std::fmt::Debug for PgPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DbPool")
            .field("name", &self.0.name())
            .field("connections", &self.connections())
            .field("idle_connections", &self.idle_connections())
            .finish_non_exhaustive()
    }
}

/// This object allows to easily interface with our PostgreSQL connection
/// which it can be from [`PgPool`] itself, the testing database object
/// or directly from [`AsyncPgConnection`].
pub enum PgConnection<'a> {
    Pooled(PooledConnection<'static, AsyncPgConnection>),
    Raw(MutexGuard<'a, AsyncPgConnection>),
}

impl std::ops::Deref for PgConnection<'_> {
    type Target = AsyncPgConnection;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Pooled(n) => n,
            Self::Raw(n) => n,
        }
    }
}

impl std::ops::DerefMut for PgConnection<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Pooled(n) => n,
            Self::Raw(n) => n,
        }
    }
}

/// Sets up the PgConnection with parameters we have in config
/// so it can behave on how it is supposed to.
#[derive(Debug)]
struct CustomDbConnector {
    readonly_mode: bool,
    statement_timeout: Duration,
}

#[async_trait]
impl CustomizeConnection<AsyncPgConnection, PoolError> for CustomDbConnector {
    async fn on_acquire(&self, conn: &mut AsyncPgConnection) -> std::result::Result<(), PoolError> {
        diesel::sql_query("SET application_name = 'capwat'")
            .execute(conn)
            .await
            .map_err(PoolError::QueryError)?;

        let timeout = self.statement_timeout.as_millis();
        diesel::sql_query(format!("SET statement_timeout = {timeout}"))
            .execute(conn)
            .await
            .map_err(PoolError::QueryError)?;

        if self.readonly_mode {
            diesel::sql_query("SET default_transaction_read_only = 't'")
                .execute(conn)
                .await
                .map_err(PoolError::QueryError)?;
        }

        Ok(())
    }
}
