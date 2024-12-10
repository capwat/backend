use async_trait::async_trait;
use capwat_error::{ApiErrorCategory, Error, Result};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl, SimpleAsyncConnection};
use diesel_async_migrations::EmbeddedMigrations;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tracing::debug;

use super::schema::{ConnectPoolInfo, DatabaseEntry};
use crate::{error::AcquireError, internal::AnyPool, pool::PgConnection};

pub struct TestPool(Option<Arc<TestPoolInner>>);

impl TestPool {
    #[tracing::instrument(name = "db.test_pool.connect")]
    pub async fn connect(migrations: &EmbeddedMigrations) -> Self {
        let mut base_url = super::DATABASE_URL.clone();
        base_url.set_path("");

        let mut url = base_url.clone();
        let connect_info = Self::setup_db(base_url.as_ref()).await;
        url.set_path(&connect_info.db_name);

        let conn = Mutex::new(Self::establish(url.as_ref()).await);
        crate::migrations::run_pending(&mut PgConnection::Raw(conn.lock().await), migrations)
            .await
            .unwrap();

        Self(Some(Arc::new(TestPoolInner {
            connect_info,
            inner: Some(conn),
        })))
    }

    async fn establish(url: &str) -> AsyncPgConnection {
        if *super::DATABASE_USE_TLS {
            crate::internal::establish_connection_with_tls(url).await
        } else {
            AsyncPgConnection::establish(url).await
        }
        .unwrap()
    }

    async fn setup_db(base_url: &str) -> ConnectPoolInfo {
        let mut conn = Self::establish(base_url).await;
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

        if super::DO_CLEANUP.swap(false, std::sync::atomic::Ordering::Relaxed) {
            Self::cleanup_test_dbs(&mut conn, now).await;
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

        drop(conn);

        ConnectPoolInfo {
            base_url: base_url.to_string(),
            db_name,
        }
    }

    async fn cleanup_test_db(conn: &mut AsyncPgConnection, name: &str) {
        diesel::sql_query(format!("DROP DATABASE IF EXISTS {name:?}"))
            .execute(conn)
            .await
            .expect("Failed to cleanup leftover testing databases");

        diesel::sql_query("DELETE FROM _capwat_test.databases WHERE name = $1")
            .bind::<diesel::sql_types::Text, _>(name)
            .execute(conn)
            .await
            .expect("Failed to cleanup leftover testing databases");
    }

    #[allow(clippy::unwrap_used)]
    async fn cleanup_test_dbs(conn: &mut AsyncPgConnection, created_before: Duration) {
        let created_before = i64::try_from(created_before.as_secs()).unwrap();

        // Risk of denial of service attack
        let delete_db_names = diesel::sql_query(
            r"
            SELECT * FROM _capwat_test.databases
            WHERE created_at < (to_timestamp($1) AT TIME ZONE 'UTC')
            ",
        )
        .bind::<diesel::sql_types::BigInt, _>(&created_before)
        .get_results::<DatabaseEntry>(conn)
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
                    .bind::<diesel::sql_types::Text, _>(db_name)
                    .execute(conn)
                    .await
                    .expect("Failed to cleanup leftover testing databases");
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
}

#[async_trait]
impl AnyPool for TestPool {
    async fn acquire(&self) -> Result<PgConnection<'_>, AcquireError> {
        let inner = self.0.as_ref().ok_or_else(|| {
            Error::unknown(AcquireError::General)
                .category(ApiErrorCategory::Outage)
                .attach_printable("Inner test pool is dropped")
        })?;

        let conn = inner.inner.as_ref().unwrap().lock().await;
        Ok(PgConnection::Raw(conn))
    }

    fn idle_connections(&self) -> u32 {
        let inner = &self.0.as_ref().unwrap().inner;
        if inner.as_ref().unwrap().try_lock().is_ok() {
            1
        } else {
            0
        }
    }

    fn connections(&self) -> u32 {
        1
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
        let Ok(mut inner) = Arc::try_unwrap(inner) else {
            return;
        };

        // We have to drop the entire test pool once it is being dropped
        // by Rust automatically. This call function `block_in_place` ensures
        // that tokio will run this and not blocking other threads.
        //
        // I did not anticipate that this single function called here it
        // works because I was very frustrated of why cleanup_test_db won't work.
        tokio::task::block_in_place(|| {
            debug!("dropping existing PostgreSQL connection");
            drop(inner.inner.take().unwrap().into_inner());

            debug!("establishing PostgreSQL connection from base url");
            let mut conn = futures::executor::block_on(AsyncPgConnection::establish(
                &inner.connect_info.base_url,
            ))
            .expect("Failed to connect to the database");

            debug!("performing testing database cleanup");
            futures::executor::block_on(Self::cleanup_test_db(
                &mut conn,
                &inner.connect_info.base_url,
            ));

            debug!("testing database cleanup done");
        });
    }
}

pub struct TestPoolInner {
    connect_info: ConnectPoolInfo,
    inner: Option<Mutex<AsyncPgConnection>>,
}
