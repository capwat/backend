use axum_test::TestServer;
use capwat_api_types::util::EncodedBase64;
use capwat_crypto::client::RegisterUserParams;
use capwat_error::ApiError;
use capwat_model::{InstanceSettings, User};
use capwat_utils::Sensitive;
use capwat_vfs::{backend::InMemoryFs, Vfs, VfsSnapshot};
use std::sync::Arc;
use tracing::{info, Instrument};

use crate::{extract::LocalInstanceSettings, App};

pub struct InitUserParams {
    pub params: RegisterUserParams,
    pub passphrase: String,
    pub user: User,
}

pub trait AsJsonResponse {
    fn as_json_error(self) -> serde_json::Value;
}

impl<T: std::fmt::Debug> AsJsonResponse for Result<T, ApiError> {
    fn as_json_error(self) -> serde_json::Value {
        match self {
            Ok(okay) => panic!("unexpected value Ok({okay:?}), expected error"),
            Err(error) => serde_json::to_value(error).unwrap(),
        }
    }
}

#[bon::builder]
pub async fn init_test_user(app: &App, name: &str, email: Option<&str>) -> InitUserParams {
    let passphrase = capwat_crypto::salt::generate_salt();
    let params = capwat_crypto::client::generate_register_user_params(&passphrase);

    let mut conn = app.db_write().await.unwrap();
    let local_settings = InstanceSettings::get_local(&mut conn).await.unwrap();
    conn.commit().await.unwrap();

    let request = crate::services::users::Register {
        name: Sensitive::new(name),
        email: email.map(Sensitive::new),
        access_key_hash: Sensitive::new(&params.access_key_hash),
        salt: Sensitive::new(&params.salt),
        symmetric_key: Sensitive::new(&params.encrypted_symmetric_key),
    };

    let result = request
        .perform(app, &LocalInstanceSettings(Arc::new(local_settings)))
        .await
        .unwrap();

    InitUserParams {
        params,
        passphrase: EncodedBase64::from_bytes(passphrase).to_string(),
        user: result.user,
    }
}

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
