use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use capwat_api_types::error::category::RegisterUserFailed;
use capwat_api_types::routes::users::{RegisterUser, RegisterUserResponse};
use capwat_error::ext::ResultExt;
use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::instance::RegistrationMode;
use capwat_model::user::InsertUser;
use capwat_model::User;
use capwat_postgres::impls::users::{InsertUserPgImpl, UserPgImpl};
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

    let access_key_hash =
        spawn_blocking(move || capwat_crypto::argon2::hash(form.access_key_hash.decode()))
            .await
            .erase_context()??;

    InsertUser::builder()
        .name(form.name.as_str())
        .maybe_email(form.email.as_ref().map(|v| v.as_str()))
        .access_key_hash(&*access_key_hash)
        .encrypted_symmetric_key(&form.symmetric_key.value().to_string())
        .salt(&form.salt.value().to_string())
        .public_key(&form.classic_keys.public.value().to_string())
        .encrypted_secret_key(&form.classic_keys.encrypted_private.value().to_string())
        .build()
        .insert(&mut conn)
        .await?;

    let response = RegisterUserResponse {
        verify_email: settings.require_email_verification,
    };

    conn.commit().await?;
    Ok((StatusCode::CREATED, Json(response)).into_response())
}

#[cfg(test)]
mod tests {
    use crate::utils::test::build_test_server;
    use axum::http::StatusCode;
    use capwat_api_types::{routes::users::RegisterUser, user::UserClassicKeys};

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_register_user() {
        let (server, _) = build_test_server().await;
        let params = capwat_crypto::client::generate_register_user_params("test");

        let body = RegisterUser::builder()
            .name("test_bot")
            .email("test@example.com")
            .access_key_hash(params.access_key_hash)
            .salt(params.salt)
            .symmetric_key(params.encrypted_symmetric_key)
            .classic_keys(
                UserClassicKeys::builder()
                    .public(params.public_key)
                    .encrypted_private(params.encrypted_secret_key)
                    .build(),
            )
            .build();

        let response = server.post("/users/register").json(&body).await;
        response.assert_status(StatusCode::CREATED);
    }
}
