use capwat_kernel::{util::env, Result};
use diesel::{migration::Migration, pg::Pg};
use diesel_async::{
    AsyncConnection, AsyncPgConnection, RunQueryDsl, SimpleAsyncConnection,
};
use once_cell::sync::Lazy;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tokio::sync::Mutex;
use url::Url;

use self::schema::ConnectPoolInfo;
use crate::internal::AsyncConnectionWrapper;

mod schema;

#[derive(Clone)]
pub struct TestPool {
    info: Option<Arc<ConnectPoolInfo>>,
    inner: Option<Arc<Mutex<AsyncPgConnection>>>,
}

impl TestPool {
    #[tracing::instrument(skip(migrations))]
    pub async fn connect(migrations: Vec<Box<dyn Migration<Pg>>>) -> Self {
        let mut base_url = Self::into_base_url(&DB_URL);
        let info = Self::setup_db(base_url.as_ref()).await;
        base_url.set_path(&info.db_name);

        Self::run_migrations(base_url.as_str(), migrations).await;

        let conn =
            Arc::new(Mutex::new(Self::establish(base_url.as_ref()).await));

        Self { info: Some(Arc::new(info)), inner: Some(conn) }
    }

    async fn run_migrations(
        url: &str,
        migrations: Vec<Box<dyn Migration<Pg>>>,
    ) {
        let conn = Self::establish(url).await;

        let mut conn = AsyncConnectionWrapper::from(conn);
        for migration in migrations {
            migration.run(&mut conn).expect("Failed to run migration");
        }
    }

    async fn setup_db(url: &str) -> schema::ConnectPoolInfo {
        let mut conn = Self::establish(url).await;

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
        .expect("Failed to initialize testing database with `DATABASE_URL`");

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System clock went backwards!");

        if DO_CLEANUP.swap(false, Ordering::SeqCst) {
            Self::cleanup_test_dbs(&mut conn, now).await;
        }

        let db_name = diesel::sql_query(
            r"
            INSERT INTO _capwat_test.databases(name)
            SELECT '_capwat_test_' || nextval('_capwat_test.database_ids')
            RETURNING *
            ",
        )
        .get_result::<schema::DatabaseEntry>(&mut conn)
        .await
        .expect("Failed to initialize testing database with `DATABASE_URL`")
        .name;

        diesel::sql_query(format!("CREATE DATABASE {db_name:?}"))
            .execute(&mut conn)
            .await
            .expect(
                "Failed to initialize testing database with `DATABASE_URL`",
            );

        drop(conn);

        schema::ConnectPoolInfo { base_url: url.to_string(), db_name }
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
    async fn cleanup_test_dbs(
        conn: &mut AsyncPgConnection,
        created_before: Duration,
    ) {
        let created_before = i64::try_from(created_before.as_secs()).unwrap();

        // Risk of denial of service attack
        let delete_db_names = diesel::sql_query(
            r"
            SELECT * FROM _capwat_test.databases
            WHERE created_at < (to_timestamp($1) AT TIME ZONE 'UTC')
            ",
        )
        .bind::<diesel::sql_types::BigInt, _>(&created_before)
        .get_results::<schema::DatabaseEntry>(conn)
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
                        &r"DELETE FROM _capwat_test.databases WHERE name = $1"
                            .to_string(),
                    )
                    .bind::<diesel::sql_types::Text, _>(db_name)
                    .execute(conn)
                    .await
                    .expect("Failed to cleanup leftover testing databases");
                },
                Err(diesel::result::Error::DatabaseError(.., e)) => {
                    eprintln!(
                        "could not clean test database {:?}: {}",
                        db_name,
                        e.message()
                    );
                },
                Err(e) => {
                    panic!("Failed to cleanup leftover testing databases:\n{e}")
                },
            }
        }
    }
}

#[derive(Debug, Error)]
#[error("Connection of test pool is dropped")]
struct TestPoolDropped;

impl TestPool {
    #[tracing::instrument(skip(self))]
    pub async fn get(&self) -> Result<super::Connection<'_>> {
        let conn = self
            .inner
            .as_ref()
            .ok_or_else(|| capwat_kernel::Error::internal(TestPoolDropped))?
            .lock()
            .await;

        Ok(super::Connection::Test(conn))
    }
}

impl TestPool {
    async fn establish(url: &str) -> AsyncPgConnection {
        if *DB_USE_TLS {
            crate::internal::establish_tls_connection(url).await
        } else {
            AsyncPgConnection::establish(url).await
        }
        .expect("Failed to connect to Postgres with `CAPWAT_DATABASE_URL`/`DATABASE_URL`")
    }

    fn into_base_url(url: &str) -> Url {
        let mut url = Url::parse(url).expect(
            "`CAPWAT_DATABASE_URL`/`DATABASE_URL` contains invalid URL",
        );

        url.set_path("");
        url
    }
}

impl Drop for TestPool {
    fn drop(&mut self) {
        let inner = self.inner.take().zip(self.info.take());
        if let Some((conn, info)) = inner {
            // Perform a shutdown!
            if Arc::strong_count(&conn) != 1 {
                return;
            }

            // Take the connection object, immediately!
            let conn = Arc::into_inner(conn)
                .expect("Failed to safely obtain owned connection object");

            tracing::info!("Dropping existing pg connection");
            drop(conn);

            tracing::info!("Establishing pg connectionn");
            let mut conn = futures::executor::block_on(
                AsyncPgConnection::establish(&info.base_url),
            )
            .expect("Failed to connect to the database");

            futures::executor::block_on(Self::cleanup_test_db(
                &mut conn,
                &info.db_name,
            ));
        }
    }
}

static DO_CLEANUP: AtomicBool = AtomicBool::new(true);

static DB_URL: Lazy<String> = Lazy::new(|| {
    env::var("CAPWAT_DB_URL")
        .and_then(|v| {
            if let Some(v) = v {
                Ok(v)
            } else {
                env::required_var_parse("DATABASE_URL")
            }
        })
        .unwrap()
});

static DB_USE_TLS: Lazy<bool> = Lazy::new(|| {
    env::var_parse("CAPWAT_DB_ENFORCE_TLS").unwrap().unwrap_or(false)
});
