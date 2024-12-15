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
