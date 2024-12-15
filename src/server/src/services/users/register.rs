use capwat_api_types::error::category::RegisterUserFailed;
use capwat_api_types::user::UserSalt;
use capwat_api_types::util::EncodedBase64;
use capwat_crypto::argon2;
use capwat_error::ext::ResultExt;
use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::instance::RegistrationMode;
use capwat_model::user::InsertUser;
use capwat_model::User;
use capwat_utils::Sensitive;
use tokio::task::spawn_blocking;

use crate::extract::LocalInstanceSettings;
use crate::App;

#[derive(Debug)]
pub struct Register<'a> {
    pub name: Sensitive<&'a str>,
    pub email: Option<Sensitive<&'a str>>,
    pub access_key_hash: Sensitive<&'a EncodedBase64>,
    pub salt: Sensitive<&'a UserSalt>,
    pub symmetric_key: Sensitive<&'a EncodedBase64>,
}

#[derive(Debug)]
pub struct RegisterResult {
    pub user: User,
}

impl Register<'_> {
    #[tracing::instrument(skip(app), name = "services.users.login")]
    pub async fn perform(
        self,
        app: &App,
        local_settings: &LocalInstanceSettings,
    ) -> Result<RegisterResult, ApiError> {
        if !app.validate_username(&self.name) {
            let error =
                ApiError::new(ApiErrorCategory::InvalidRequest).message("Invalid username.");

            return Err(error);
        }

        if let Some(email) = self.email.as_deref() {
            if !app.validate_email(email) {
                return Err(ApiError::new(ApiErrorCategory::InvalidRequest)
                    .message("Invalid email address."));
            }
        }

        if let RegistrationMode::Closed = local_settings.registration_mode {
            return Err(ApiError::new(ApiErrorCategory::RegisterUserFailed(
                RegisterUserFailed::Closed,
            )));
        }

        let mut conn = app.db_write().await?;
        if User::check_username_taken(&mut conn, &self.name).await? {
            return Err(ApiError::new(ApiErrorCategory::RegisterUserFailed(
                RegisterUserFailed::UsernameTaken,
            )));
        }

        if let Some(email) = self.email.as_deref() {
            if User::check_email_taken(&mut conn, email).await? {
                return Err(ApiError::new(ApiErrorCategory::RegisterUserFailed(
                    RegisterUserFailed::EmailTaken,
                )));
            }
        } else if local_settings.require_email_registration {
            return Err(ApiError::new(ApiErrorCategory::RegisterUserFailed(
                RegisterUserFailed::EmailRequired,
            )));
        }

        let access_key_hash = self.access_key_hash.decode().to_vec();
        let access_key_hash = spawn_blocking(move || argon2::hash(access_key_hash))
            .await
            .erase_context()??;

        let user = InsertUser::builder()
            .name(&self.name)
            .maybe_email(self.email.as_ref().map(|v| v.value()).map(|v| &**v))
            .access_key_hash(&access_key_hash)
            .encrypted_symmetric_key(&self.symmetric_key.encode())
            .salt(&self.salt.value().to_string())
            .build()
            .insert(&mut conn)
            .await?;

        conn.commit().await.erase_context()?;

        Ok(RegisterResult { user })
    }
}
