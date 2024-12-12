use crate::{extract::LocalInstanceSettings, App};
use axum_test::TestServer;
use capwat_api_types::{user::UserSalt, util::EncodedBase64};
use capwat_error::ApiError;
use capwat_model::InstanceSettings;
use capwat_utils::Sensitive;
use capwat_vfs::{backend::InMemoryFs, Vfs, VfsSnapshot};
use std::fmt::Debug;
use tracing::{info, Instrument};

pub trait TestResultExt {
    fn expect_error_json(self) -> serde_json::Value;
}

impl<T: Debug> TestResultExt for Result<T, ApiError> {
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

#[tracing::instrument(name = "test_utils.build_test_app")]
pub async fn build_test_app() -> (App, InstanceSettings) {
    let imfs = InMemoryFs::new();
    let vfs = Vfs::new(imfs.apply_snapshot("/", VfsSnapshot::empty_dir()).unwrap());
    let _ = capwat_utils::env::load_dotenv(&Vfs::new_std());

    capwat_tracing::init_for_tests();
    capwat_db::install_error_middleware();

    let span = tracing::info_span!("test.build_app");
    async {
        let app = App::new_for_tests(vfs).await;

        let mut conn = app.db_write().await.unwrap();
        let settings = InstanceSettings::setup_local(&mut conn).await.unwrap();
        conn.commit().await.unwrap();

        (app, settings)
    }
    .instrument(span)
    .await
}

#[tracing::instrument(name = "test_utils.build_test_server")]
pub async fn build_test_server() -> (TestServer, App, InstanceSettings) {
    let span = tracing::info_span!("test.build_server");
    let (app, settings) = build_test_app().instrument(span.clone()).await;

    async {
        let router = crate::build_axum_router(app.clone());
        let server = TestServer::new(router).unwrap();

        info!("Test server is now running");
        (server, app, settings)
    }
    .instrument(span)
    .await
}
