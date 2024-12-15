use axum_test::TestServer;
use capwat_db::testing::test_with_pool;
use capwat_error::ext::ResultExt;
use capwat_error::Result;
use capwat_model::InstanceSettings;
use capwat_vfs::backend::InMemoryFs;
use capwat_vfs::{Vfs, VfsSnapshot};
use std::future::Future;
use thiserror::Error;
use tracing::{info, Instrument};

use crate::App;

#[allow(async_fn_in_trait)]
pub trait TestFn {
    async fn run_test(self, path: &'static str) -> Result<()>;
}

impl<Fut> TestFn for fn(App) -> Fut
where
    Fut: Future<Output = Result<()>>,
{
    async fn run_test(self, path: &'static str) -> Result<()> {
        initialize_with_app(path, |app, _| self(app)).await
    }
}

impl<Fut> TestFn for fn(App, InstanceSettings) -> Fut
where
    Fut: Future<Output = Result<()>>,
{
    async fn run_test(self, path: &'static str) -> Result<()> {
        initialize_with_app(path, self).await
    }
}

impl<Fut> TestFn for fn(App, LocalInstanceSettings) -> Fut
where
    Fut: Future<Output = Result<()>>,
{
    async fn run_test(self, path: &'static str) -> Result<()> {
        initialize_with_app(path, |app, settings| {
            self(app, LocalInstanceSettings::new(settings))
        })
        .await
    }
}

impl<Fut> TestFn for fn(TestServer) -> Fut
where
    Fut: Future<Output = Result<()>>,
{
    async fn run_test(self, path: &'static str) -> Result<()> {
        test_with_server(path, |_, _, server| self(server)).await
    }
}

impl<Fut> TestFn for fn(App, TestServer) -> Fut
where
    Fut: Future<Output = Result<()>>,
{
    async fn run_test(self, path: &'static str) -> Result<()> {
        test_with_server(path, |app, _, server| self(app, server)).await
    }
}

async fn test_with_server<
    F: Future<Output = Result<()>>,
    C: FnOnce(App, InstanceSettings, TestServer) -> F,
>(
    path: &'static str,
    callback: C,
) -> F::Output {
    #[derive(Debug, Error)]
    #[error("Unable to initialize test server")]
    struct TestServerFailed;

    let span = tracing::info_span!("test.build_server");
    initialize_with_app(path, |app, settings| {
        async move {
            let router = crate::build_axum_router(app.clone());
            let server = TestServer::new(router)
                .map_err(|_| capwat_error::Error::unknown_generic(TestServerFailed))?;

            info!("test server is running");
            callback(app, settings, server).await
        }
        .instrument(span)
    })
    .await
}

static JWT_PRIVATE_KEY: &[u8] =
    include_bytes!(concat!(env!("CARGO_WORKSPACE_DIR"), "/tests/files/jwt.pem"));

async fn initialize_with_app<
    F: Future<Output = Result<()>>,
    C: FnOnce(App, InstanceSettings) -> F,
>(
    path: &'static str,
    callback: C,
) -> F::Output {
    let span = tracing::info_span!("test.build_app");
    let future = test_with_pool(path, &capwat_model::DB_MIGRATIONS, |pool| async {
        let imfs = InMemoryFs::new();
        let vfs = Vfs::new(
            imfs.apply_snapshot(
                "/",
                VfsSnapshot::build_dir()
                    .file("jwt.pem", JWT_PRIVATE_KEY)
                    .build(),
            )?,
        );

        let app = App::new_for_tests(pool.into(), vfs);

        let mut conn = app.db_write().await?;
        let settings = InstanceSettings::setup_local(&mut conn).await?;
        conn.commit().await.erase_context()?;

        callback(app, settings).await
    });
    future.instrument(span).await
}

/////////////////////////////////////////////////////////////////////////////////////////
use crate::extract::LocalInstanceSettings;
use capwat_api_types::{user::UserSalt, util::EncodedBase64};
use capwat_error::ApiError;
use capwat_utils::Sensitive;
use std::fmt::Debug;

pub trait TestResultExt {
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

pub mod local_instance {
    use super::*;

    #[tracing::instrument(skip(app), name = "test.local_instance.get_settings")]
    pub async fn get_settings(app: &App) -> LocalInstanceSettings {
        let inner = InstanceSettings::get_local(&mut app.db_read().await.unwrap())
            .await
            .unwrap();

        LocalInstanceSettings::new(inner)
    }
}

pub mod users {
    use super::*;
    use crate::extract::SessionUser;
    use capwat_model::{id::UserId, User};

    pub struct TestUserSession {
        // it can be used later on when we're going to
        // implement E2EE in Capwat
        pub credientials: Credentials,
        pub user: User,
        pub token: String,
    }

    impl TestUserSession {
        #[tracing::instrument(skip_all, name = "test_utils.users.get_session_user")]
        pub async fn get_session_user(&self, app: &App) -> SessionUser {
            SessionUser::from_db(&mut app.db_read().await.unwrap(), self.user.id)
                .await
                .unwrap()
        }
    }

    #[bon::builder]
    #[tracing::instrument(skip(app, server), name = "test_utils.users.start_session")]
    pub async fn start_server_session(
        app: &App,
        server: &mut TestServer,
        name: &str,
        email: Option<&str>,
    ) -> TestUserSession {
        let session = start_session()
            .app(app)
            .name(name)
            .maybe_email(email)
            .call()
            .await;

        server.add_header("authorization", format!("Bearer {}", session.token));
        session
    }

    #[bon::builder]
    #[tracing::instrument(skip(app), name = "test_utils.users.start_session")]
    pub async fn start_session(app: &App, name: &str, email: Option<&str>) -> TestUserSession {
        let credientials = register()
            .app(app)
            .name(name)
            .maybe_email(email)
            .call()
            .await;

        let local_settings = local_instance::get_settings(app).await;
        let request = crate::services::users::Login {
            name_or_email: Sensitive::new(name),
            access_key_hash: Some(Sensitive::new(&credientials.access_key_hash)),
        };

        let response = request.perform(app, &local_settings).await.unwrap();
        TestUserSession {
            credientials,
            user: response.user,
            token: response.token,
        }
    }

    pub struct Credentials {
        pub access_key_hash: EncodedBase64,
        pub passphrase: String,
        pub salt: UserSalt,
        pub user_id: UserId,
    }

    #[bon::builder]
    #[tracing::instrument(skip(app), name = "test_utils.users.register")]
    pub async fn register(app: &App, name: &str, email: Option<&str>) -> Credentials {
        let passphrase = capwat_crypto::salt::generate_salt();
        let params = capwat_crypto::client::generate_register_user_params(&passphrase);
        let local_settings = local_instance::get_settings(app).await;

        let request = crate::services::users::Register {
            name: Sensitive::new(name),
            email: email.map(Sensitive::new),
            access_key_hash: Sensitive::new(&params.access_key_hash),
            salt: Sensitive::new(&params.salt),
            symmetric_key: Sensitive::new(&params.encrypted_symmetric_key),
        };

        let response = request.perform(app, &local_settings).await.unwrap();

        Credentials {
            access_key_hash: params.access_key_hash,
            passphrase: EncodedBase64::from_bytes(passphrase).to_string(),
            salt: params.salt,
            user_id: response.user.id,
        }
    }
}
