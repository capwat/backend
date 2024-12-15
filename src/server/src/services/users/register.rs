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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{self, TestResultExt};

    use assert_json_diff::assert_json_include;
    use capwat_model::InstanceSettings;
    use serde_json::json;

    #[capwat_macros::api_test]
    async fn should_register(app: App, local_settings: LocalInstanceSettings) {
        let params = test_utils::generate_register_user_params("alice").await;

        let request = Register {
            name: Sensitive::new("alice"),
            email: Some(Sensitive::new("alice@example.com")),
            access_key_hash: Sensitive::new(&params.access_key_hash),
            salt: Sensitive::new(&params.salt),
            symmetric_key: Sensitive::new(&params.encrypted_symmetric_key),
        };

        request.perform(&app, &local_settings).await.unwrap();

        let user = User::find_by_login(&mut app.db_read().await.unwrap(), "alice")
            .await
            .unwrap();

        assert!(user.is_some());

        let user = user.unwrap();
        assert_eq!(user.name, "alice");
        assert_eq!(user.email.as_deref(), Some("alice@example.com"));
        assert_ne!(user.access_key_hash, params.access_key_hash.encode());
        assert_eq!(user.salt, params.salt.to_string());
        assert_eq!(
            user.encrypted_symmetric_key,
            params.encrypted_symmetric_key.to_string()
        );
    }

    #[capwat_macros::api_test]
    async fn should_reject_if_email_is_taken(app: App, local_settings: LocalInstanceSettings) {
        test_utils::users::register()
            .name("alice")
            .email("alice@example.com")
            .app(&app)
            .call()
            .await;

        let params = test_utils::generate_register_user_params("bob").await;
        let request = Register {
            name: Sensitive::new("bob"),
            email: Some(Sensitive::new("alice@example.com")),
            access_key_hash: Sensitive::new(&params.access_key_hash),
            salt: Sensitive::new(&params.salt),
            symmetric_key: Sensitive::new(&params.encrypted_symmetric_key),
        };

        let error = request
            .perform(&app, &local_settings)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "register_user_failed",
                "subcode": "email_taken",
            }),
        );
    }

    #[capwat_macros::api_test]
    async fn should_reject_if_username_is_taken(app: App, local_settings: LocalInstanceSettings) {
        test_utils::users::register()
            .name("alice")
            .app(&app)
            .call()
            .await;

        let params = test_utils::generate_register_user_params("alice").await;
        let request = Register {
            name: Sensitive::new("alice"),
            email: None,
            access_key_hash: Sensitive::new(&params.access_key_hash),
            salt: Sensitive::new(&params.salt),
            symmetric_key: Sensitive::new(&params.encrypted_symmetric_key),
        };

        let error = request
            .perform(&app, &local_settings)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "register_user_failed",
                "subcode": "username_taken",
            }),
        );
    }

    #[capwat_macros::api_test]
    async fn should_reject_if_email_is_not_present_but_required(app: App) {
        let params = test_utils::generate_register_user_params("alice").await;

        let request = Register {
            name: Sensitive::new("alice"),
            email: None,
            access_key_hash: Sensitive::new(&params.access_key_hash),
            salt: Sensitive::new(&params.salt),
            symmetric_key: Sensitive::new(&params.encrypted_symmetric_key),
        };

        let local_settings = LocalInstanceSettings::new(
            InstanceSettings::builder()
                .require_email_registration(true)
                .build(),
        );

        let error = request
            .perform(&app, &local_settings)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "register_user_failed",
                "subcode": "email_required",
            }),
        );
    }

    #[capwat_macros::api_test]
    async fn should_reject_if_registration_is_closed(app: App) {
        let params = test_utils::generate_register_user_params("alice").await;

        let request = Register {
            name: Sensitive::new("alice"),
            email: None,
            access_key_hash: Sensitive::new(&params.access_key_hash),
            salt: Sensitive::new(&params.salt),
            symmetric_key: Sensitive::new(&params.encrypted_symmetric_key),
        };

        let local_settings = LocalInstanceSettings::new(
            InstanceSettings::builder()
                .registration_mode(RegistrationMode::Closed)
                .build(),
        );

        let error = request
            .perform(&app, &local_settings)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "register_user_failed",
                "subcode": "closed",
            }),
        );
    }

    #[capwat_macros::api_test]
    async fn should_reject_if_invalid_username(app: App, local_settings: LocalInstanceSettings) {
        let params = test_utils::generate_register_user_params("alice").await;

        let request = Register {
            name: Sensitive::new(""),
            email: None,
            access_key_hash: Sensitive::new(&params.access_key_hash),
            salt: Sensitive::new(&params.salt),
            symmetric_key: Sensitive::new(&params.encrypted_symmetric_key),
        };

        let error = request
            .perform(&app, &local_settings)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "invalid_request",
                "message": "Invalid username.",
            }),
        );
    }

    #[capwat_macros::api_test]
    async fn should_reject_if_invalid_email(app: App, local_settings: LocalInstanceSettings) {
        let params = test_utils::generate_register_user_params("alice").await;

        let request = Register {
            name: Sensitive::new("alice"),
            email: Some(Sensitive::new("")),
            access_key_hash: Sensitive::new(&params.access_key_hash),
            salt: Sensitive::new(&params.salt),
            symmetric_key: Sensitive::new(&params.encrypted_symmetric_key),
        };

        let error = request
            .perform(&app, &local_settings)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "invalid_request",
                "message": "Invalid email address.",
            }),
        );
    }
}
