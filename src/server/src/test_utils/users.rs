use axum::http::header;
use axum_test::TestServer;
use capwat_api_types::{user::UserSalt, util::EncodedBase64};
use capwat_model::{id::UserId, User};
use capwat_utils::Sensitive;

use crate::{extract::SessionUser, App};

/// It contains user's credentials to log in to a Capwat instance
/// or encrypt/decrypt data from various sources using their
/// public and private keys.
///
/// This is served as a simulation of what a client should behave
/// when dealing with the Capwat API.
#[allow(unused)]
pub struct Credentials {
    pub access_key_hash: EncodedBase64,
    pub encrypted_symmetric_key: EncodedBase64,
    pub passphrase: String,
    pub salt: UserSalt,
    pub user_id: UserId,
}

#[allow(unused)]
pub struct UserSessionData {
    // It can be used later on if we're going to implement
    // E2EE system in Capwat.
    pub credentials: Credentials,
    pub user: User,
    /// User's login token
    pub token: String,
}

impl UserSessionData {
    /// Gets the [`SessionUser`] extractor.
    #[tracing::instrument(skip_all, name = "tes_utils.users.get_session_user", fields(
        user.id = %self.user.id,
        user.name = %self.user.name,
    ))]
    pub async fn get_session_user(&self, app: &App) -> SessionUser {
        SessionUser::from_db(&mut app.db_read().await.unwrap(), self.user.id)
            .await
            .unwrap()
    }
}

#[bon::builder]
#[tracing::instrument(skip(app, server), name = "test_utils.users.override_credentials")]
pub async fn override_credentials(
    app: &App,
    server: &mut TestServer,
    name: &str,
    email: Option<&str>,
) -> UserSessionData {
    let session = get_session_data()
        .app(app)
        .name(name)
        .maybe_email(email)
        .call()
        .await;

    server.add_header(header::AUTHORIZATION, format!("Bearer {}", session.token));
    session
}

#[bon::builder]
#[tracing::instrument(skip(app), name = "test_utils.users.start_session")]
pub async fn get_session_data(app: &App, name: &str, email: Option<&str>) -> UserSessionData {
    let credentials = register()
        .app(app)
        .name(name)
        .maybe_email(email)
        .call()
        .await;

    let local_settings = super::local_settings::from_local(app).await;
    let request = crate::services::users::Login {
        name_or_email: Sensitive::new(name),
        access_key_hash: Some(Sensitive::new(&credentials.access_key_hash)),
    };

    let response = request.perform(app, &local_settings).await.unwrap();
    UserSessionData {
        credentials,
        user: response.user,
        token: response.token,
    }
}

#[bon::builder]
#[tracing::instrument(skip(app), name = "test_utils.users.register")]
pub async fn register(app: &App, name: &str, email: Option<&str>) -> Credentials {
    let passphrase = capwat_crypto::salt::generate_salt();
    let params = capwat_crypto::client::generate_register_user_params(&passphrase);
    let local_settings = super::local_settings::from_local(app).await;

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
        encrypted_symmetric_key: params.encrypted_symmetric_key,
        passphrase: EncodedBase64::from_bytes(passphrase).to_string(),
        salt: params.salt,
        user_id: response.user.id,
    }
}
