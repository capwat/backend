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

use crate::extract::{Json, LocalInstanceSettings, RequiresCaptcha};
use crate::utils::users::{hash_password, validate_password};
use crate::App;

#[tracing::instrument(skip(app, _requires_captcha), name = "v1.users.register")]
pub async fn register(
    app: App,
    _requires_captcha: RequiresCaptcha,
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

    if !validate_password(&form.password) {
        return Err(ApiError::new(ApiErrorCategory::RegisterUserFailed(
            RegisterUserFailed::InvalidPassword,
        )));
    }

    // We don't need to have some kind of constant time equals operation since
    // we're registering for an account anyway not logging in but we're going to
    // replace this with let the user generate their own password hash.
    if form.password != form.password_verify {
        return Err(ApiError::new(ApiErrorCategory::RegisterUserFailed(
            RegisterUserFailed::UnmatchedPassword,
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

    let password_hash = spawn_blocking(move || hash_password(&form.password))
        .await
        .erase_context()??;

    InsertUser::builder()
        .name(form.name.as_str())
        .maybe_email(form.email.as_ref().map(|v| v.as_str()))
        .password_hash(&*password_hash)
        .build()
        .create(&mut conn)
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
    use capwat_api_types::{error::category::RegisterUserFailed, routes::users::RegisterUser};
    use capwat_error::{ApiError, ApiErrorCategory};
    use capwat_model::instance_settings::{
        InstanceSettings, RegistrationMode, UpdateInstanceSettings,
    };
    use capwat_postgres::queries::instance_settings::InstanceSettingsPgImpl;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_register_user() {
        let (server, _) = build_test_server().await;
        let body = RegisterUser::builder()
            .name("test_bot")
            .email("test@example.com")
            .password("super_fluffy_unicorns")
            .password_verify("super_fluffy_unicorns")
            .build();

        let response = server.post("/users/register").json(&body).await;
        response.assert_status(StatusCode::CREATED);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_reject_if_registration_is_closed() {
        let (server, app) = build_test_server().await;

        let mut conn = app.db_write().await.unwrap();
        let form = UpdateInstanceSettings::builder()
            .registration_mode(RegistrationMode::Closed)
            .build();

        InstanceSettings::update_local(&mut conn, &form)
            .await
            .unwrap();

        conn.commit().await.unwrap();

        let body = RegisterUser::builder()
            .name("test_bot")
            .email("test@example.com")
            .password("super_fluffy_unicorns")
            .password_verify("super_fluffy_unicorns")
            .build();

        let response = server.post("/users/register").json(&body).await;
        response.assert_status(StatusCode::BAD_REQUEST);
        response.assert_json_contains(&ApiError::new(ApiErrorCategory::RegisterUserFailed(
            RegisterUserFailed::Closed,
        )));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_reject_if_username_or_email_is_taken() {
        let (server, _) = build_test_server().await;
        let body = RegisterUser::builder()
            .name("test_bot")
            .email("test@example.com")
            .password("super_fluffy_unicorns")
            .password_verify("super_fluffy_unicorns")
            .build();

        let response = server.post("/users/register").json(&body).await;
        response.assert_status(StatusCode::CREATED);

        let response = server.post("/users/register").json(&body).await;
        response.assert_status(StatusCode::BAD_REQUEST);
        response.assert_json_contains(&ApiError::new(ApiErrorCategory::RegisterUserFailed(
            RegisterUserFailed::UsernameTaken,
        )));

        let body = RegisterUser::builder()
            .name("test_bot2")
            .email("test@example.com")
            .password("super_fluffy_unicorns")
            .password_verify("super_fluffy_unicorns")
            .build();

        let response = server.post("/users/register").json(&body).await;
        response.assert_status(StatusCode::BAD_REQUEST);
        response.assert_json_contains(&ApiError::new(ApiErrorCategory::RegisterUserFailed(
            RegisterUserFailed::EmailTaken,
        )));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_reject_if_invalid_password() {
        let (server, _) = build_test_server().await;
        let body = RegisterUser::builder()
            .name("test_bot")
            .email("test@example.com")
            .password("sup")
            .password_verify("super_fluffy_unicorns")
            .build();

        let response = server.post("/users/register").json(&body).await;
        response.assert_status(StatusCode::BAD_REQUEST);
        response.assert_json_contains(&ApiError::new(ApiErrorCategory::RegisterUserFailed(
            RegisterUserFailed::InvalidPassword,
        )));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_reject_if_unmatched_password() {
        let (server, _) = build_test_server().await;
        let body = RegisterUser::builder()
            .name("test_bot")
            .email("test@example.com")
            .password("super_fluffy_unicorns")
            .password_verify("super_fluffy_unicorn")
            .build();

        let response = server.post("/users/register").json(&body).await;
        response.assert_status(StatusCode::BAD_REQUEST);
        response.assert_json_contains(&ApiError::new(ApiErrorCategory::RegisterUserFailed(
            RegisterUserFailed::UnmatchedPassword,
        )));
    }
}
