use axum::response::{IntoResponse, Response};
use capwat_api_types::encrypt::ClassicKey;
use capwat_api_types::error::category::{AccessKeyRequiredInfo, LoginUserFailed};
use capwat_api_types::routes::users::{LoginUser, LoginUserResponse};
use capwat_api_types::user::{UserClassicKeys, UserSalt};
use capwat_api_types::util::EncodedBase64;
use capwat_crypto::argon2;
use capwat_crypto::future::SubtleTimingFutureExt;
use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::user::UserKeys;
use capwat_model::User;
use capwat_postgres::impls::{UserKeysPgImpl, UserPgImpl};
use chrono::Utc;
use std::str::FromStr;
use std::time::Duration;
use tokio::task::spawn_blocking;

use crate::auth::jwt;
use crate::controllers::checkers::{check_email_status, check_user_keys};
use crate::extract::{Json, LocalInstanceSettings};
use crate::App;

#[tracing::instrument(skip(app), name = "v1.users.login")]
pub async fn login(
    app: App,
    LocalInstanceSettings(settings): LocalInstanceSettings,
    Json(form): Json<LoginUser>,
) -> Result<Response, ApiError> {
    let mut conn = app.db_read_prefer_primary().await?;
    let now = Utc::now();

    // This is actually a great strategy to get the spammer rate-limited easily
    // since we need 2 HTTP requests to login successfully.
    let user = User::find_by_login(&mut conn, &form.name_or_email).await?;
    let user = async {
        let salt = if let Some(ref user) = user {
            UserSalt::from_str(&user.salt).attach_printable("got an invalid user salt")?
        } else {
            capwat_crypto::salt::generate_user_salt()
        };

        let Some((user, input_hash)) = user.zip(form.access_key_hash.as_deref()) else {
            // We should not give away the user that the user does not exists.
            return Err(ApiError::new(ApiErrorCategory::LoginUserFailed(
                if form.access_key_hash.is_some() {
                    LoginUserFailed::InvalidCredientials
                } else {
                    let info = AccessKeyRequiredInfo { salt };
                    LoginUserFailed::AccessKeyRequired(info)
                },
            )));
        };

        let input_hash = input_hash.decode().to_vec();
        let correct_hash = user.access_key_hash.to_string();

        let is_password_matched =
            spawn_blocking(move || argon2::verify(&input_hash, &correct_hash))
                .await
                .erase_context()??;

        if !is_password_matched {
            return Err(ApiError::new(ApiErrorCategory::LoginUserFailed(
                LoginUserFailed::InvalidCredientials,
            )));
        }

        Ok::<_, ApiError>(user)
    }
    .subtle_timing(Duration::from_secs(1))
    .await?;

    let their_keys = UserKeys::get_current(&mut conn, user.id)
        .await
        .attach_printable("could not get user current keys")?;

    check_email_status(&user, &settings)?;
    check_user_keys(&their_keys, now)?;

    let token = jwt::LoginClaims::generate(&user, &["application"]).encode(&app)?;
    let response = Json(LoginUserResponse {
        token,
        name: user.name,
        display_name: user.display_name,
        email_verified: Some(user.email_verified),
        classic_keys: UserClassicKeys {
            public: ClassicKey::from_str(&their_keys.public_key)
                .attach_printable("could not parse user's classic public key")?
                .into(),
            // panics are handled in the middleware so we don't need to worry about it
            encrypted_private: EncodedBase64::from_encoded(&their_keys.encrypted_secret_key)
                .unwrap()
                .into(),
        },
        encrypted_symmetric_key: user.encrypted_symmetric_key,
    });

    Ok(response.into_response())
}
