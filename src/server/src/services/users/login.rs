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
                        LoginUserFailed::InvalidCredientials
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
                    LoginUserFailed::InvalidCredientials,
                )));
            }

            Ok::<_, ApiError>(user)
        }
        .subtle_timing(Duration::from_secs(1))
        .await?;

        check_email_status(&user, &local_settings)?;

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
    use crate::util::test::AsJsonResponse;
    use assert_json_diff::assert_json_include;
    use capwat_model::instance::UpdateInstanceSettings;
    use serde_json::json;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_login() {
        let (app, settings) = crate::util::test::build_test_app().await;
        let alice_params = crate::util::test::init_test_user()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let local_settings = LocalInstanceSettings::new(settings);
        let response = Login {
            name_or_email: Sensitive::new("Alice"),
            access_key_hash: Some(Sensitive::new(&alice_params.params.access_key_hash)),
        }
        .perform(&app, &local_settings)
        .await;

        assert!(response.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_reject_if_user_has_no_email() {
        let (app, _) = crate::util::test::build_test_app().await;
        let alice_params = crate::util::test::init_test_user()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let mut conn = app.db_write().await.unwrap();
        let settings = UpdateInstanceSettings::builder()
            .require_email_registration(true)
            .build()
            .perform_local(&mut conn)
            .await
            .unwrap();

        conn.commit().await.unwrap();

        let local_settings = LocalInstanceSettings::new(settings);
        let error = Login {
            name_or_email: Sensitive::new("Alice"),
            access_key_hash: Some(Sensitive::new(&alice_params.params.access_key_hash)),
        }
        .perform(&app, &local_settings)
        .await
        .as_json_error();

        assert_json_include!(
            actual: error,
            expected: json!({ "code": "no_email_address" })
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_reject_if_user_has_not_verified_their_email() {
        let (app, _) = crate::util::test::build_test_app().await;
        let alice_params = crate::util::test::init_test_user()
            .app(&app)
            .name("alice")
            .email("alice@example.com")
            .call()
            .await;

        let mut conn = app.db_write().await.unwrap();
        let settings = UpdateInstanceSettings::builder()
            .require_email_verification(true)
            .build()
            .perform_local(&mut conn)
            .await
            .unwrap();

        conn.commit().await.unwrap();

        let local_settings = LocalInstanceSettings::new(settings);
        let error = Login {
            name_or_email: Sensitive::new("Alice"),
            access_key_hash: Some(Sensitive::new(&alice_params.params.access_key_hash)),
        }
        .perform(&app, &local_settings)
        .await
        .as_json_error();

        assert_json_include!(
            actual: error,
            expected: json!({ "code": "email_verification_required" })
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_reject_if_gave_invalid_access_key() {
        let (app, settings) = crate::util::test::build_test_app().await;
        crate::util::test::init_test_user()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let local_settings = LocalInstanceSettings::new(settings);
        let error = Login {
            name_or_email: Sensitive::new("Alice"),
            access_key_hash: Some(Sensitive::new(&EncodedBase64::from_bytes(b""))),
        }
        .perform(&app, &local_settings)
        .await
        .as_json_error();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "login_user_failed",
                "subcode": "invalid_credientials",
            })
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_throw_invalid_creds_if_user_not_found_but_access_key_is_present() {
        let (app, settings) = crate::util::test::build_test_app().await;

        let local_settings = LocalInstanceSettings::new(settings);
        let error = Login {
            name_or_email: Sensitive::new("Alice"),
            access_key_hash: Some(Sensitive::new(&EncodedBase64::from_bytes(b""))),
        }
        .perform(&app, &local_settings)
        .await
        .as_json_error();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "login_user_failed",
                "subcode": "invalid_credientials",
            }),
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_give_random_salt_if_user_is_not_found() {
        let (app, settings) = crate::util::test::build_test_app().await;

        let local_settings = LocalInstanceSettings::new(settings);
        let error = Login {
            name_or_email: Sensitive::new("Alice"),
            access_key_hash: None,
        }
        .perform(&app, &local_settings)
        .await
        .as_json_error();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "login_user_failed",
                "subcode": "access_key_required",
            }),
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_give_their_salt_if_user_is_found() {
        let (app, settings) = crate::util::test::build_test_app().await;
        let user = crate::util::test::init_test_user()
            .app(&app)
            .name("alice")
            .email("alice@example.com")
            .call()
            .await;

        let local_settings = LocalInstanceSettings::new(settings);
        let error = Login {
            // lowercase testing :)
            name_or_email: Sensitive::new("Alice"),
            access_key_hash: None,
        }
        .perform(&app, &local_settings)
        .await
        .as_json_error();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "login_user_failed",
                "subcode": "access_key_required",
                "data": {
                    "salt": user.params.salt,
                },
            })
        );
    }
}
