use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use capwat_api_types::error::category::RegisterUserFailed;
use capwat_api_types::routes::users::{RegisterUser, RegisterUserResponse};
use capwat_error::ext::ResultExt;
use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::instance_settings::RegistrationMode;
use capwat_model::user::InsertUser;
use capwat_model::User;
use capwat_postgres::queries::users::{InsertUserPgImpl, UserPgImpl};
use tokio::task::spawn_blocking;

use crate::extract::{Json, LocalInstanceSettings};
use crate::App;

#[tracing::instrument(skip(app), name = "v1.users.register")]
pub async fn register(
    app: App,
    LocalInstanceSettings(settings): LocalInstanceSettings,
    Json(form): Json<RegisterUser>,
) -> Result<Response, ApiError> {
    if let RegistrationMode::Closed = settings.registration_mode {
        return Err(ApiError::new(ApiErrorCategory::RegisterUserFailed(
            RegisterUserFailed::Closed,
        )));
    }

    let mut conn = app.db_write().await?;
    if User::check_username_taken(&mut conn, &form.name).await? {
        return Err(ApiError::new(ApiErrorCategory::RegisterUserFailed(
            RegisterUserFailed::UsernameTaken,
        )));
    }

    if let Some(email) = form.email.as_deref() {
        if User::check_email_taken(&mut conn, email).await? {
            return Err(ApiError::new(ApiErrorCategory::RegisterUserFailed(
                RegisterUserFailed::EmailTaken,
            )));
        }
    } else if settings.require_email_registration {
        return Err(ApiError::new(ApiErrorCategory::RegisterUserFailed(
            RegisterUserFailed::EmailRequired,
        )));
    }

    // We need to hash it twice because we'll going to calculate
    // our own SHA256 of access key anyway.
    let access_key_hash =
        spawn_blocking(move || capwat_crypto::argon2::hash(&form.access_key_hash))
            .await
            .erase_context()??;

    InsertUser::builder()
        .name(form.name.as_str())
        .maybe_email(form.email.as_ref().map(|v| v.as_str()))
        .access_key_hash(&*access_key_hash)
        .root_classic_pk(&*form.classic_keys.public.value().to_string())
        .root_encrypted_classic_sk(&form.classic_keys.encrypted_private.value())
        .root_pqc_pk(&*form.pqc_keys.public.value().to_string())
        .root_encrypted_pqc_sk(&*form.pqc_keys.encrypted_private)
        .build()
        .create(&mut conn)
        .await?;

    let response = RegisterUserResponse {
        verify_email: settings.require_email_verification,
    };

    conn.commit().await?;
    Ok((StatusCode::CREATED, Json(response)).into_response())
}

//     if !validate_password(&form.password) {
//         return Err(ApiError::new(ApiErrorCategory::RegisterUserFailed(
//             RegisterUserFailed::InvalidPassword,
//         )));
//     }

//     // We don't need to have some kind of constant time equals operation since
//     // we're registering for an account anyway not logging in but we're going to
//     // replace this with let the user generate their own password hash.
//     if form.password != form.password_verify {
//         return Err(ApiError::new(ApiErrorCategory::RegisterUserFailed(
//             RegisterUserFailed::UnmatchedPassword,
//         )));
//     }

//     if let Some(email) = form.email.as_deref() {
//         if User::check_email_taken(&mut conn, email).await? {
//             return Err(ApiError::new(ApiErrorCategory::RegisterUserFailed(
//                 RegisterUserFailed::EmailTaken,
//             )));
//         }
//     } else if settings.require_email_registration {
//         return Err(ApiError::new(ApiErrorCategory::RegisterUserFailed(
//             RegisterUserFailed::EmailRequired,
//         )));
//     }

//     let password_hash = spawn_blocking(move || hash_password(&form.password))
//         .await
//         .erase_context()??;

//     InsertUser::builder()
//         .name(form.name.as_str())
//         .maybe_email(form.email.as_ref().map(|v| v.as_str()))
//         .password_hash(&*password_hash)
//         .build()
//         .create(&mut conn)
//         .await?;

//     let response = RegisterUserResponse {
//         verify_email: settings.require_email_verification,
//     };

//     conn.commit().await?;

//     Ok((StatusCode::CREATED, Json(response)).into_response())
// }

#[cfg(test)]
mod tests {
    use crate::utils::test::build_test_server;
    use axum::http::StatusCode;
    use capwat_api_types::{
        error::category::RegisterUserFailed,
        routes::users::RegisterUser,
        users::{UserClassicKeys, UserPostQuantumKeys, UserSalt},
    };
    use capwat_crypto::{curve25519, ml_kem768};
    use capwat_error::{ApiError, ApiErrorCategory};
    use capwat_model::instance_settings::{
        InstanceSettings, RegistrationMode, UpdateInstanceSettings,
    };
    use capwat_postgres::queries::instance_settings::InstanceSettingsPgImpl;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_register_user() {
        let (server, _) = build_test_server().await;

        let (pk, sk) = curve25519::KeyPair::generate().split();
        let (pqc_pk, pqc_sk) = ml_kem768::KeyPair::generate().split();
        let salt = UserSalt::from(capwat_crypto::generate_salt());

        let body = RegisterUser::builder()
            .name("test_bot")
            .email("test@example.com")
            .access_key_hash(format!("wokeoekwopk1231po2321"))
            .salt(salt.into())
            .classic_keys(
                UserClassicKeys::builder()
                    .encrypted_private(sk.serialize().into())
                    .public(pk.serialize())
                    .build()
                    .into(),
            )
            .pqc_keys(
                UserPostQuantumKeys::builder()
                    .encrypted_private(pqc_sk.serialize().into())
                    .public(pqc_pk.serialize())
                    .build()
                    .into(),
            )
            .build();

        let response = server.post("/users/register").json(&body).await;
        response.assert_status(StatusCode::CREATED);
    }
}
