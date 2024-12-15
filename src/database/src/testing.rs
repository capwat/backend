use sqlx::migrate::Migrator;
use sqlx::testing::{TestArgs, TestSupport};
use sqlx::{ConnectOptions, Connection, Postgres};
use std::future::Future;
use std::process::Termination;
use std::time::Duration;
use tracing::warn;

use crate::pool::{PgPool, PgPooledConnection};

#[allow(async_fn_in_trait)]
pub trait TestFn {
    type Output: Termination;

    async fn run_test(self, path: &'static str, migrator: &'static Migrator) -> Self::Output;
}

impl<Fut> TestFn for fn(PgPool) -> Fut
where
    Fut: Future,
    Fut::Output: Termination,
{
    type Output = Fut::Output;

    async fn run_test(self, path: &'static str, migrator: &'static Migrator) -> Self::Output {
        test_with_pool(path, migrator, |pool| self(pool.into())).await
    }
}

impl<Fut> TestFn for fn(PgPooledConnection) -> Fut
where
    Fut: Future,
    Fut::Output: Termination,
{
    type Output = Fut::Output;

    async fn run_test(self, path: &'static str, migrator: &'static Migrator) -> Self::Output {
        test_with_pool(path, migrator, |pool| async move {
            let conn = pool.acquire().await.expect("could not acquire connection");
            self(conn).await
        })
        .await
    }
}

pub async fn test_with_pool<F: Future, C: FnOnce(sqlx::PgPool) -> F>(
    path: &'static str,
    migrator: &'static Migrator,
    callback: C,
) -> F::Output {
    let sqlx_args = TestArgs {
        test_path: path,
        migrator: Some(migrator),
        fixtures: &[],
    };

    let test_context = Postgres::test_context(&sqlx_args)
        .await
        .expect("failed to setup test database");

    let mut conn = test_context
        .connect_opts
        .connect()
        .await
        .expect("failed to connect to the test database");

    migrator
        .run_direct(&mut conn)
        .await
        .expect("failed to apply migrations");

    conn.close()
        .await
        .expect("failed to close setup connection");

    let pool = test_context
        .pool_opts
        .connect_with(test_context.connect_opts)
        .await
        .expect("failed to connect to test pool");

    let result = callback(pool.clone()).await;

    let close_timed_out = tokio::time::timeout(Duration::from_secs(10), pool.close())
        .await
        .is_err();

    if close_timed_out {
        warn!("test {path} held onto Pool after exiting");
    }

    result
}
