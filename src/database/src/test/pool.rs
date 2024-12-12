use async_trait::async_trait;
use capwat_config::{DatabasePool, DatabasePools};
use capwat_error::{ApiErrorCategory, Error, Result};
use capwat_utils::ProtectedString;
use diesel_async::{RunQueryDsl, SimpleAsyncConnection};
use diesel_async_migrations::EmbeddedMigrations;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, warn};
use url::Url;

use super::schema::{ConnectPoolInfo, DatabaseEntry};

use crate::error::AcquireError;
use crate::internal::AnyPool;
use crate::pool::PgConnection;
use crate::PgPool;

pub struct TestPool(Option<Arc<TestPoolInner>>);

pub struct TestPoolInner {
    connect_info: ConnectPoolInfo,
    pool: PgPool,
}

impl TestPool {
    #[tracing::instrument(skip_all, name = "db.test_pool.connect")]
    pub async fn connect(migrations: &EmbeddedMigrations) -> Self {
        let base_url = Self::get_base_db_url(&super::DATABASE_URL);
        let connect_info = Self::setup_temporary_db(&base_url).await;

        let primary_cfg = DatabasePool {
            min_connections: 1,
            max_connections: 5,
            readonly_mode: false,
            url: ProtectedString::new(base_url.join(&connect_info.db_name).unwrap()),
        };

        let pool = PgPool::build(
            &DatabasePools {
                primary: primary_cfg.clone(),
                replica: None,
                enforce_tls: *super::DATABASE_USE_TLS,
                idle_timeout: Duration::from_secs(600),

                // do this ASAP
                connection_timeout: Duration::from_secs(1),
                statement_timeout: Duration::from_secs(5),
            },
            &primary_cfg,
        );

        crate::migrations::run_pending(&mut pool.acquire().await.unwrap(), migrations)
            .await
            .unwrap();

        pool.check_health(None)
            .await
            .expect("database is unhealthy to continue the rest of the tests. Please check your PostgreSQL database connection before trying to perform the tests again");

        Self(Some(Arc::new(TestPoolInner { connect_info, pool })))
    }

    #[tracing::instrument(skip_all, name = "db.test_pool.setup_temporary_db")]
    async fn setup_temporary_db(base_url: &Url) -> ConnectPoolInfo {
        debug!("setting up test database");

        let pool = {
            let primary_cfg = DatabasePool {
                min_connections: 30,
                max_connections: 200,
                readonly_mode: false,
                url: ProtectedString::new(base_url),
            };
            PgPool::build(
                &DatabasePools {
                    primary: primary_cfg.clone(),
                    replica: None,
                    enforce_tls: *super::DATABASE_USE_TLS,
                    // do this ASAP
                    connection_timeout: Duration::from_secs(1),
                    idle_timeout: Duration::from_secs(600),
                    statement_timeout: Duration::from_secs(1),
                },
                &primary_cfg,
            )
        };

        let master_pool = match super::MASTER_POOL.try_insert(pool) {
            Ok(inserted) => inserted,
            Err((existing, _)) => existing,
        };

        // Make sure the master pool has reached the minimum connections as possible
        loop {
            master_pool.check_health(None).await.unwrap();
            if master_pool.connections() > 30 {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        let mut conn = master_pool.acquire().await.unwrap();
        conn.batch_execute(
            r"
                LOCK TABLE pg_catalog.pg_namespace IN SHARE ROW EXCLUSIVE MODE;
                CREATE SCHEMA IF NOT EXISTS _capwat_test;
                CREATE TABLE IF NOT EXISTS _capwat_test.databases (
                    id int primary key generated always as identity,
                    name text not null,
                    created_at timestamp not null default now()
                );

                CREATE INDEX IF NOT EXISTS databases_created_at
                    ON _capwat_test.databases(created_at);

                CREATE SEQUENCE IF NOT EXISTS _capwat_test.database_ids
                ",
        )
        .await
        .expect("failed to initialize testing database");

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System clock went backwards!");

        if super::DO_CLEANUP.swap(false, std::sync::atomic::Ordering::SeqCst) {
            Self::cleanup_test_dbs(&mut PgConnection::Raw(&mut conn), now).await;
        }

        let db_name = diesel::sql_query(
            r"
            INSERT INTO _capwat_test.databases(name)
            SELECT '_capwat_test_' || nextval('_capwat_test.database_ids')
            RETURNING *
            ",
        )
        .get_result::<DatabaseEntry>(&mut conn)
        .await
        .expect("failed to initialize testing database")
        .name;

        diesel::sql_query(format!("CREATE DATABASE {db_name:?}"))
            .execute(&mut conn)
            .await
            .expect("failed to initialize testing database");

        ConnectPoolInfo {
            base_url: base_url.to_string(),
            db_name,
        }
    }

    #[tracing::instrument(skip_all, name = "db.test_pool.cleanup_test_dbs")]
    #[allow(clippy::unwrap_used)]
    async fn cleanup_test_dbs(conn: &mut PgConnection<'_>, created_before: Duration) {
        debug!("cleaning unused test databases...");
        let created_before = i64::try_from(created_before.as_secs()).unwrap();

        // Risk of denial of service attack
        let delete_db_names = diesel::sql_query(
            r"
            SELECT * FROM _capwat_test.databases
            WHERE created_at < (to_timestamp($1) AT TIME ZONE 'UTC')
            ",
        )
        .bind::<diesel::sql_types::BigInt, _>(&created_before)
        .get_results::<DatabaseEntry>(&mut *conn)
        .await
        .expect("Failed to cleanup leftover testing databases")
        .into_iter()
        .map(|v| v.name)
        .collect::<Vec<_>>();

        if delete_db_names.is_empty() {
            return;
        }

        let mut command = String::new();
        for db_name in delete_db_names {
            use std::fmt::Write;
            command.clear();
            writeln!(command, "DROP DATABASE IF EXISTS {db_name:?}").ok();
            match conn.batch_execute(&command).await {
                Ok(..) => {
                    diesel::sql_query(
                        r"DELETE FROM _capwat_test.databases WHERE name = $1".to_string(),
                    )
                    .bind::<diesel::sql_types::Text, _>(&db_name)
                    .execute(conn)
                    .await
                    .expect("Failed to cleanup leftover testing databases");

                    debug!("cleaned unused test database: {db_name:?}");
                }
                Err(diesel::result::Error::DatabaseError(.., e)) => {
                    eprintln!(
                        "could not clean test database {:?}: {}",
                        db_name,
                        e.message()
                    );
                }
                Err(e) => {
                    panic!("Failed to cleanup leftover testing databases:\n{e}")
                }
            }
        }
    }

    async fn cleanup_test_db(conn: &mut PgConnection<'_>, name: &str) {
        diesel::sql_query(format!("DROP DATABASE IF EXISTS {name:?}"))
            .execute(&mut *conn)
            .await
            .expect("Failed to cleanup leftover testing databases");

        diesel::sql_query("DELETE FROM _capwat_test.databases WHERE name = $1")
            .bind::<diesel::sql_types::Text, _>(name)
            .execute(&mut *conn)
            .await
            .expect("Failed to cleanup leftover testing databases");
    }
}

impl TestPool {
    fn get_base_db_url(database_url: &Url) -> Url {
        let mut base_url = database_url.clone();
        base_url.set_path("");
        base_url
    }
}

#[async_trait]
impl AnyPool for TestPool {
    async fn acquire(&self) -> Result<PgConnection<'_>, AcquireError> {
        let inner = self.0.as_ref().ok_or_else(|| {
            Error::unknown(AcquireError::General)
                .category(ApiErrorCategory::Outage)
                .attach_printable("Inner test pool is dropped")
        })?;
        debug_assert!(!inner.pool.0.is_testing());

        inner.pool.acquire().await
    }

    fn idle_connections(&self) -> u32 {
        let pool = &self.0.as_ref().unwrap().pool;
        debug_assert!(!pool.0.is_testing());
        pool.idle_connections()
    }

    fn connections(&self) -> u32 {
        let pool = &self.0.as_ref().unwrap().pool;
        debug_assert!(!pool.0.is_testing());
        pool.connections()
    }

    fn is_testing(&self) -> bool {
        true
    }
}

impl Drop for TestPool {
    #[tracing::instrument(skip(self), name = "db.test_pool.on_drop")]
    fn drop(&mut self) {
        let Some(inner) = self.0.take() else {
            return;
        };

        // Perform a shutdown!
        let Ok(inner) = Arc::try_unwrap(inner) else {
            return;
        };

        // We have to drop the entire test pool once it is being dropped
        // by Rust automatically. This call function `block_in_place` ensures
        // that tokio will run this and not blocking other threads.
        //
        // I did not anticipate that this single function called here it
        // works because I was very frustrated of why cleanup_test_db won't work.
        tokio::task::block_in_place(|| {
            debug!("dropping existing PostgreSQL pool");
            drop(inner.pool);

            // this is very illegal, using tokio and futures executor
            // at the same time... -_-
            futures::executor::block_on(async move {
                debug!("acquiring connection from the master pool");

                let mut conn = super::MASTER_POOL.get().unwrap().acquire().await.unwrap();
                debug!("performing testing database cleanup");

                Self::cleanup_test_db(&mut conn, &inner.connect_info.base_url).await;
            });

            debug!("testing database cleanup done");
        });
    }
}
