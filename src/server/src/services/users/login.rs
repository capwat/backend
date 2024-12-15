use capwat_api_types::error::category::{AccessKeyRequiredInfo, LoginUserFailed};
use capwat_api_types::user::UserSalt;
use capwat_api_types::util::EncodedBase64;
use capwat_crypto::argon2;
use capwat_crypto::future::SubtleTimingFutureExt;
use capwat_error::ext::ResultExt;
use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::User;
use capwat_utils::Sensitive;
use std::str::FromStr;
use std::time::Duration;
use tokio::task::spawn_blocking;

use crate::auth::jwt::LoginClaims;
use crate::extract::LocalInstanceSettings;
use crate::services::util::check_email_status;
use crate::App;

#[derive(Debug)]
pub struct Login<'a> {
    pub name_or_email: Sensitive<&'a str>,
    pub access_key_hash: Option<Sensitive<&'a EncodedBase64>>,
}

impl Login<'_> {
    #[tracing::instrument(skip(app), name = "services.users.login")]
    pub async fn perform(
        self,
        app: &App,
        local_settings: &LocalInstanceSettings,
    ) -> Result<LoginResponse, ApiError> {
        let mut conn = app.db_read_prefer_primary().await?;

        // TODO: Randomize subtle timing duration
        //
        // This is actually a great strategy to avoid spam or brute-force because
        // it can be rate-limited easily since we need 2 HTTP requests to login
        // successfully.
        let user = User::find_by_login(&mut conn, &self.name_or_email).await?;
        let user = async {
            let salt = if let Some(ref user) = user {
                UserSalt::from_str(&user.salt)
                    .attach_printable("got an invalid user salt from the database")?
            } else {
                capwat_crypto::salt::generate_user_salt()
            };

            let Some((user, input_hash)) = user.zip(self.access_key_hash.as_deref()) else {
                // We should not give away the user that the user does not exists.
                return Err(ApiError::new(ApiErrorCategory::LoginUserFailed(
                    if self.access_key_hash.is_some() {
                        LoginUserFailed::InvalidCredentials
                    } else {
                        let info = AccessKeyRequiredInfo { salt };
                        LoginUserFailed::AccessKeyRequired(info)
                    },
                )));
            };

            let input_hash = input_hash.decode().to_vec();
            let correct_hash = user.access_key_hash.to_string();

            let is_matched = spawn_blocking(move || argon2::verify(&input_hash, &correct_hash))
                .await
                .erase_context()??;

            if !is_matched {
                return Err(ApiError::new(ApiErrorCategory::LoginUserFailed(
                    LoginUserFailed::InvalidCredentials,
                )));
            }

            Ok::<_, ApiError>(user)
        }
        .subtle_timing(Duration::from_secs(1))
        .await?;

        check_email_status(&user, local_settings)?;

        let token = LoginClaims::generate(app, &user, None, None).encode(app)?;
        Ok(LoginResponse { user, token })
    }
}

#[derive(Debug)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::jwt::Scope;
    use crate::test_utils::{self, TestResultExt};

    use assert_json_diff::assert_json_include;
    use capwat_model::InstanceSettings;
    use serde_json::json;

    #[capwat_macros::api_test]
    async fn should_login(app: App, local_settings: LocalInstanceSettings) {
        let credentials = test_utils::users::register()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let request = Login {
            name_or_email: Sensitive::new("alice"),
            access_key_hash: Some(Sensitive::new(&credentials.access_key_hash)),
        };

        let response = request.perform(&app, &local_settings).await.unwrap();
        assert_eq!(
            Some(&response.user),
            User::find_by_login(&mut app.db_write().await.unwrap(), "alice")
                .await
                .unwrap()
                .as_ref()
        );

        let result = LoginClaims::decode(&app, &response.token);
        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.scope, Scope::APPLICATION);
        assert_eq!(claims.sub, response.user.id.0);
    }

    #[capwat_macros::api_test]
    async fn should_login_with_any_cases_of_entry(app: App, local_settings: LocalInstanceSettings) {
        let credentials = test_utils::users::register()
            .app(&app)
            .name("alice")
            .email("alice@example.com")
            .call()
            .await;

        let request = Login {
            name_or_email: Sensitive::new("AlicE"),
            access_key_hash: Some(Sensitive::new(&credentials.access_key_hash)),
        };

        let response = request.perform(&app, &local_settings).await.unwrap();
        assert_eq!(
            Some(&response.user),
            User::find_by_login(&mut app.db_write().await.unwrap(), "alice")
                .await
                .unwrap()
                .as_ref()
        );

        let result = LoginClaims::decode(&app, &response.token);
        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.scope, Scope::APPLICATION);
        assert_eq!(claims.sub, response.user.id.0);

        let request = Login {
            name_or_email: Sensitive::new("Alice@Example.com"),
            access_key_hash: Some(Sensitive::new(&credentials.access_key_hash)),
        };

        let response = request.perform(&app, &local_settings).await.unwrap();
        assert_eq!(
            Some(&response.user),
            User::find_by_login(&mut app.db_write().await.unwrap(), "alice")
                .await
                .unwrap()
                .as_ref()
        );

        let result = LoginClaims::decode(&app, &response.token);
        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.scope, Scope::APPLICATION);
        assert_eq!(claims.sub, response.user.id.0);
    }

    #[capwat_macros::api_test]
    async fn should_reject_if_user_has_no_email_if_email_required(app: App) {
        let credentials = test_utils::users::register()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let local_settings = LocalInstanceSettings::new(
            InstanceSettings::builder()
                .require_email_registration(true)
                .build(),
        );

        let request = Login {
            name_or_email: Sensitive::new("alice"),
            access_key_hash: Some(Sensitive::new(&credentials.access_key_hash)),
        };

        let error = request
            .perform(&app, &local_settings)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "no_email_address",
            }),
        );
    }

    #[capwat_macros::api_test]
    async fn should_reject_if_user_has_not_verified_their_email_if_required(app: App) {
        let credentials = test_utils::users::register()
            .app(&app)
            .name("alice")
            .email("alice@example.com")
            .call()
            .await;

        let local_settings = LocalInstanceSettings::new(
            InstanceSettings::builder()
                .require_email_verification(true)
                .build(),
        );

        let request = Login {
            name_or_email: Sensitive::new("alice"),
            access_key_hash: Some(Sensitive::new(&credentials.access_key_hash)),
        };

        let error = request
            .perform(&app, &local_settings)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "email_verification_required",
            }),
        );
    }

    #[capwat_macros::api_test]
    async fn should_reject_if_user_gave_invalid_access_key(
        app: App,
        local_settings: LocalInstanceSettings,
    ) {
        test_utils::users::register()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let access_key_hash = &EncodedBase64::from_bytes(b"");
        let request = Login {
            name_or_email: Sensitive::new("Alice"),
            access_key_hash: Some(Sensitive::new(access_key_hash)),
        };

        let error = request
            .perform(&app, &local_settings)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "login_user_failed",
                "subcode": "invalid_credentials",
            }),
        );
    }

    #[capwat_macros::api_test]
    async fn should_throw_invalid_credentials_if_user_not_found_but_access_key_is_present(
        app: App,
        local_settings: LocalInstanceSettings,
    ) {
        let access_key_hash = &EncodedBase64::from_bytes(b"");
        let request = Login {
            name_or_email: Sensitive::new("Alice"),
            access_key_hash: Some(Sensitive::new(access_key_hash)),
        };

        let error = request
            .perform(&app, &local_settings)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "login_user_failed",
                "subcode": "invalid_credentials",
            }),
        );
    }

    #[capwat_macros::api_test]
    async fn should_give_random_user_salt_if_user_is_not_found(
        app: App,
        local_settings: LocalInstanceSettings,
    ) {
        let request = Login {
            name_or_email: Sensitive::new("Alice"),
            access_key_hash: None,
        };

        let error = request
            .perform(&app, &local_settings)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "login_user_failed",
                "subcode": "access_key_required",
                "data": {},
            }),
        );
    }

    #[capwat_macros::api_test]
    async fn should_give_their_salt_if_user_is_found(
        app: App,
        local_settings: LocalInstanceSettings,
    ) {
        let alice = test_utils::users::register()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let request = Login {
            name_or_email: Sensitive::new("Alice"),
            access_key_hash: None,
        };

        let error = request
            .perform(&app, &local_settings)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "login_user_failed",
                "subcode": "access_key_required",
                "data": {
                    "salt": alice.salt,
                },
            }),
        );
    }
}
