use capwat_kernel::error::ext::{ErrorExt3, ResultExt};
use capwat_kernel::{util::Sensitive, Result};
use diesel::{migration::Migration, pg::Pg};
use diesel_async::{
    pooled_connection::{AsyncDieselConnectionManager, ManagerConfig},
    AsyncConnection,
};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use self::error::MigrationError;
use self::internal::AsyncConnectionWrapper;

mod any;
mod config;
mod connection;
mod error;
mod internal;
mod test;
mod transaction;

pub use any::AnyPool;
pub use config::PoolConfig;
pub use connection::Connection;
pub use error::PoolError;
pub use internal::{PgConnection, PooledConn};
pub use test::TestPool;
pub use transaction::{Transaction, TransactionBuilder};

pub mod prelude;

#[derive(Clone)]
pub struct Pool(Arc<dyn AnyPool>);

impl Pool {
    #[tracing::instrument(skip_all)]
    pub async fn connect(
        cfg: &PoolConfig,
        enforce_tls: bool,
        timeout: Duration,
        migrations: Vec<Box<dyn Migration<Pg>>>,
    ) -> Result<Self> {
        Self::run_migrations(cfg.url.clone(), enforce_tls, migrations).await?;

        let manager = if enforce_tls {
            let mut config = ManagerConfig::default();
            config.custom_setup = Box::new(internal::establish_tls_connection);

            AsyncDieselConnectionManager::new_with_config(
                cfg.url.as_str(),
                config,
            )
        } else {
            AsyncDieselConnectionManager::new(cfg.url.as_str())
        };

        // deadpool builder will throw error if runtime is not specified.
        let inner = internal::Pool::builder(manager)
            .max_size(cfg.pool_size())
            .create_timeout(Some(timeout))
            .runtime(deadpool::Runtime::Tokio1)
            .build()
            .expect("Unexpected error from deadpool");

        Ok(Self(Arc::new(inner)))
    }

    #[tracing::instrument(skip_all)]
    pub async fn connect_for_tests(
        migrations: Vec<Box<dyn Migration<Pg>>>,
    ) -> Self {
        let pool = test::TestPool::connect(migrations).await;
        pool.wait_until_healthy(None)
            .await
            .expect("Failed to respond to the database");

        Self(Arc::new(pool))
    }
}

impl Pool {
    // TODO: Make Pool::run_migrations not block current running async thread
    //
    // To make it to async friendly (simply require Send in Migration trait),
    // all diesel packages need to depend with our modified version of diesel.
    #[tracing::instrument(skip_all)]
    pub async fn run_migrations(
        url: Sensitive<String>,
        enforce_tls: bool,
        migrations: Vec<Box<dyn Migration<Pg>>>,
    ) -> Result<()> {
        // Safely run migrations from there
        let conn = if enforce_tls {
            internal::establish_tls_connection(url.as_str())
        } else {
            internal::PgConnection::establish(url.as_str())
        }
        .await?;

        let mut conn = AsyncConnectionWrapper::from(conn);
        for migration in migrations {
            migration
                .run(&mut conn)
                .into_error()
                .change_context(MigrationError)?;
        }

        Ok(())
    }
}

impl Pool {
    #[must_use]
    pub fn connections(&self) -> usize {
        1
    }

    #[tracing::instrument(skip(self))]
    pub async fn begin_default(&mut self) -> Result<Transaction<'_>> {
        let conn = self.get().await?;
        TransactionBuilder::new(conn).build().await
    }

    #[tracing::instrument(skip(self))]
    pub async fn begin(&self) -> Result<TransactionBuilder<'_>> {
        let conn = self.get().await?;
        Ok(TransactionBuilder::new(conn))
    }

    #[tracing::instrument(skip(self))]
    pub async fn get(&self) -> Result<Connection<'_>> {
        self.0.get().await
    }

    #[tracing::instrument(skip(self))]
    pub async fn wait_until_healthy(
        &self,
        timeout: Option<Duration>,
    ) -> Result<bool> {
        self.0.wait_until_healthy(timeout).await
    }
}

impl Debug for Pool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pool")
            .field("type", &self.0.name())
            .field("connections", &self.0.connections())
            .finish()
    }
}
