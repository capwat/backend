use async_trait::async_trait;
use capwat_kernel::error::ext::{ErrorExt, IntoError, ResultExt};
use capwat_kernel::error::Result;
use diesel_async::RunQueryDsl;
use std::time::Duration;

use crate::{error::PoolError, internal, test, Connection};

#[async_trait]
pub trait AnyPool: Send + Sync + 'static {
    async fn get(&self) -> Result<Connection<'_>>;
    async fn wait_until_healthy(
        &self,
        timeout: Option<Duration>,
    ) -> Result<bool>;

    fn connections(&self) -> usize;
    fn name(&self) -> &'static str;
}

#[async_trait]
impl AnyPool for test::TestPool {
    #[tracing::instrument(skip(self))]
    async fn get(&self) -> Result<Connection<'_>> {
        Self::get(self).await
    }

    #[tracing::instrument(skip(self))]
    async fn wait_until_healthy(
        &self,
        timeout: Option<Duration>,
    ) -> Result<bool> {
        let mut conn = Self::get(&self).await?;

        // TODO: Check if it is safe to cancel this query by canceling the query future.
        if let Some(timeout) = timeout {
            let result = tokio::time::timeout(
                timeout,
                diesel::sql_query("SELECT 1;").execute(&mut conn),
            )
            .await;

            match result {
                Ok(Err(err)) => Err(err.into_error()),
                Ok(..) => Ok(true),
                Err(..) => Ok(false),
            }
        } else {
            Ok(true)
        }
    }

    fn connections(&self) -> usize {
        1
    }

    fn name(&self) -> &'static str {
        "TestPool"
    }
}

#[async_trait]
impl AnyPool for internal::Pool {
    #[tracing::instrument(skip(self), fields(
        connections = %self.connections(),
        is_closed = ?self.is_closed(),
        max_connections = %self.status().max_size,
    ))]
    async fn get(&self) -> Result<Connection<'_>> {
        match Self::get(self).await {
            Ok(conn) => Ok(Connection::Pool(conn)),
            Err(err) if self.connections() == 0 => {
                Err(err).into_error().change_context(PoolError::UnhealthyPool)
            },
            Err(err) => {
                Err(err).into_error().change_context(PoolError::General)
            },
        }
    }

    #[tracing::instrument(skip(self), fields(
        connections = %self.connections(),
        is_closed = ?self.is_closed(),
        max_connections = %self.status().max_size,
    ))]
    async fn wait_until_healthy(
        &self,
        timeout: Option<Duration>,
    ) -> Result<bool> {
        let defaults = self.timeouts();
        let timeouts = deadpool::managed::Timeouts {
            create: timeout.or(defaults.create),
            wait: timeout.or(defaults.wait),
            recycle: timeout.or(defaults.recycle),
        };
        match self.timeout_get(&timeouts).await {
            Ok(..) => Ok(true),
            Err(deadpool::managed::PoolError::Timeout(..)) => Ok(false),
            Err(e) => Err(e).into_error().change_context(PoolError::General),
        }
    }

    fn connections(&self) -> usize {
        self.status().size
    }

    fn name(&self) -> &'static str {
        "Pool"
    }
}
