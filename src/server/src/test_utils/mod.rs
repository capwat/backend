use crate::extract::LocalInstanceSettings;
use crate::App;

use axum_test::TestServer;
use capwat_crypto::client::RegisterUserParams;
use capwat_db::testing::test_with_pool;
use capwat_error::ext::ResultExt;
use capwat_error::ApiError;
use capwat_model::InstanceSettings;
use capwat_utils::cache::MapCache;
use capwat_vfs::backend::InMemoryFs;
use capwat_vfs::{Vfs, VfsSnapshot};
use std::fmt::Debug;
use std::future::Future;
use std::sync::LazyLock;
use std::time::Duration;
use thiserror::Error;
use tokio::task::spawn_blocking;
use tracing::{info, Instrument};

pub mod local_settings;
pub mod users;

// Rust tests are done asynchronously on a same environment/static values
// so we have no choice but to find ways to make it faster.
/// This function behaves like [`generate_register_user_params`] found in [`capwat_crypto`]
/// but it caches registration parameters to perform tests somewhat faster.
pub async fn generate_register_user_params(passphrase: &'static str) -> RegisterUserParams {
    static DATA: LazyLock<MapCache<String, RegisterUserParams>> = LazyLock::new(|| {
        MapCache::builder()
            .time_to_live(Duration::from_days(1))
            .max_capacity(5)
            .build()
    });

    if let Some(existing) = DATA.get(passphrase).await {
        existing
    } else {
        spawn_blocking(move || {
            let new_data =
                capwat_crypto::client::generate_register_user_params(passphrase.as_bytes());

            let _ = DATA.insert(passphrase.to_string(), new_data.clone());
            new_data
        })
        .await
        .unwrap()
    }
}

pub trait TestResultExt {
    /// This allows to redirectly serialize into [`serde_json::Value`]
    /// from [`Capwat API error`].
    ///
    /// ## Panics
    /// It will panic if depending on the implementation of that trait.
    ///
    /// For example, when used with [`Result<T, ApiError>`], it will throw
    /// an error if the result is [`Ok`].
    ///
    /// [`Capwat API error`]: ApiError
    fn expect_error_json(self) -> serde_json::Value;
}

impl<T: Debug> TestResultExt for std::result::Result<T, ApiError> {
    fn expect_error_json(self) -> serde_json::Value {
        match self {
            Ok(okay) => panic!("unexpected value Ok({okay:?}), expected error"),
            Err(error) => serde_json::to_value(error).unwrap(),
        }
    }
}

// Implementation of #[capwat_macros::server_test] suite
#[doc(hidden)]
#[allow(async_fn_in_trait)]
pub trait TestFn {
    async fn run_test(self, path: &'static str);
}

impl<Fut> TestFn for fn(App) -> Fut
where
    Fut: Future<Output = ()>,
{
    async fn run_test(self, path: &'static str) {
        initialize_with_app(path, |app, _| self(app)).await
    }
}

impl<Fut> TestFn for fn(App, InstanceSettings) -> Fut
where
    Fut: Future<Output = ()>,
{
    async fn run_test(self, path: &'static str) {
        initialize_with_app(path, self).await
    }
}

impl<Fut> TestFn for fn(App, LocalInstanceSettings) -> Fut
where
    Fut: Future<Output = ()>,
{
    async fn run_test(self, path: &'static str) {
        initialize_with_app(path, |app, settings| {
            self(app, LocalInstanceSettings::new(settings))
        })
        .await
    }
}

impl<Fut> TestFn for fn(TestServer) -> Fut
where
    Fut: Future<Output = ()>,
{
    async fn run_test(self, path: &'static str) {
        test_with_server(path, |_, _, server| self(server)).await
    }
}

impl<Fut> TestFn for fn(App, TestServer) -> Fut
where
    Fut: Future<Output = ()>,
{
    async fn run_test(self, path: &'static str) {
        test_with_server(path, |app, _, server| self(app, server)).await
    }
}

async fn test_with_server<
    F: Future<Output = ()>,
    C: FnOnce(App, InstanceSettings, TestServer) -> F,
>(
    path: &'static str,
    callback: C,
) {
    #[derive(Debug, Error)]
    #[error("Unable to initialize test server")]
    struct TestServerFailed;

    let span = tracing::info_span!("test.build_server");
    initialize_with_app(path, |app, settings| {
        async move {
            let router = crate::build_axum_router(app.clone());
            let server = TestServer::new(router)
                .map_err(|_| capwat_error::Error::unknown_generic(TestServerFailed))
                .unwrap();

            info!("test server is running");
            callback(app, settings, server).await
        }
        .instrument(span)
    })
    .await
}

static JWT_PRIVATE_KEY: &[u8] =
    include_bytes!(concat!(env!("CARGO_WORKSPACE_DIR"), "/tests/files/jwt.pem"));

async fn initialize_with_app<F: Future<Output = ()>, C: FnOnce(App, InstanceSettings) -> F>(
    path: &'static str,
    callback: C,
) {
    let span = tracing::info_span!("test.build_app");
    let future = test_with_pool(path, &capwat_model::DB_MIGRATIONS, |pool| async {
        let imfs = InMemoryFs::new();
        let vfs = Vfs::new(
            imfs.apply_snapshot(
                "/",
                VfsSnapshot::build_dir()
                    .file("jwt.pem", JWT_PRIVATE_KEY)
                    .build(),
            )
            .unwrap(),
        );

        let app = App::new_for_tests(pool.into(), vfs);

        let mut conn = app.db_write().await.unwrap();
        let settings = InstanceSettings::setup_local(&mut conn).await.unwrap();
        conn.commit().await.erase_context().unwrap();

        callback(app, settings).await
    });
    future.instrument(span).await
}
