use capwat_diesel::{Connection, Pool as DieselPool};
use capwat_diesel::{Transaction, TransactionBuilder};
use capwat_kernel::error::ext::ErrorExt3;
use capwat_kernel::util::future::{CapwatFutureExt, IntoOptionalFuture};
use capwat_kernel::Result;
use diesel::migration::MigrationSource;

mod config;
pub use config::DatabaseConfig;

#[derive(Debug, Clone)]
pub struct Database {
    primary: DieselPool,
    read_only_replica: Option<DieselPool>,
}

impl Database {
    #[tracing::instrument]
    pub async fn connect(cfg: &DatabaseConfig) -> Result<Self> {
        DieselPool::run_migrations(
            cfg.primary.connection_url().clone(),
            cfg.enforces_tls(),
            crate::MIGRATIONS.migrations().into_error()?,
        )
        .await?;

        let (primary_elapsed, primary) = DieselPool::connect(
            &cfg.primary,
            cfg.enforces_tls(),
            cfg.timeout(),
        )
        .benchmark()
        .await;

        let primary = primary?;
        if primary.wait_until_healthy(None).await? {
            tracing::info!(
                elapsed = ?primary_elapsed,
                "Successfully connected to primary database"
            );
        } else {
            tracing::info!("Primary database connection is unhealthy");
        }

        let read_only_replica = if let Some(pool_cfg) = cfg.replica() {
            let (elapsed, pool) = DieselPool::connect(
                pool_cfg,
                cfg.enforces_tls(),
                cfg.timeout(),
            )
            .benchmark()
            .await;

            let pool = pool?;
            if pool.wait_until_healthy(None).await? {
                tracing::info!(
                    ?elapsed,
                    "Successfully connected to replica database"
                );
            } else {
                tracing::info!("Replica database connection is unhealthy");
            }

            Some(pool)
        } else {
            None
        };

        Ok(Self { primary, read_only_replica })
    }

    #[tracing::instrument]
    pub async fn from_pools(
        primary: DieselPool,
        read_only_replica: Option<DieselPool>,
    ) -> Result<Self> {
        let (elapsed, primary_healthy) =
            primary.wait_until_healthy(None).benchmark().await;

        if primary_healthy? {
            tracing::info!(
                ?elapsed,
                "Successfully connected to primary database"
            );
        } else {
            tracing::info!("Primary database connection is unhealthy");
        }

        if let Some(replica) = read_only_replica.as_ref() {
            let (elapsed, replica_healthy) =
                replica.wait_until_healthy(None).benchmark().await;

            let replica_healthy = replica_healthy?;
            if replica_healthy {
                tracing::info!(
                    ?elapsed,
                    "Successfully connected to replica database"
                );
            } else {
                tracing::info!("Replica database connection is unhealthy");
            }
        }

        Ok(Self { primary, read_only_replica })
    }
}

impl Database {
    #[tracing::instrument]
    pub async fn read(&self) -> Result<Connection<'_>> {
        let read_only_conn =
            self.read_only_replica.as_ref().map(|v| v.get()).optional().await;

        match read_only_conn {
            Some(Ok(conn)) => Ok(conn),
            Some(Err(err)) if is_unhealthy(&err) => self.primary.get().await,
            Some(Err(err)) => Err(err),
            None => self.primary.get().await,
        }
    }

    #[tracing::instrument]
    pub async fn read_prefer_primary(&self) -> Result<Connection<'_>> {
        match (self.primary.get().await, self.read_only_replica.as_ref()) {
            (Ok(conn), ..) => Ok(conn),
            (Err(err), Some(pool)) if is_unhealthy(&err) => pool.get().await,
            (Err(err), ..) => Err(err),
        }
    }

    #[tracing::instrument]
    pub async fn write(&self) -> Result<TransactionBuilder<'_>> {
        self.primary.begin().await
    }

    #[tracing::instrument]
    pub async fn write_defaults(&self) -> Result<Transaction<'_>> {
        self.primary.begin().await?.build().await
    }
}

fn is_unhealthy(error: &capwat_kernel::Error) -> bool {
    matches!(
        error.downcast_ref::<capwat_diesel::PoolError>(),
        Some(capwat_diesel::PoolError::UnhealthyPool)
    )
}
