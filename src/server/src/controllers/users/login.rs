use axum::response::{IntoResponse, Response};
use capwat_api_types::error::category::LoginUserFailed;
use capwat_api_types::routes::users::LoginUser;
use capwat_crypto::future::SubtleTimerFutureExt;
use capwat_error::ext::ResultExt;
use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::instance_settings::InstanceSettings;
use capwat_model::User;
use capwat_postgres::queries::users::UserPgImpl;
use serde_json::json;
use std::time::Duration;
use tokio::task::spawn_blocking;

use crate::extract::{Json, LocalInstanceSettings, RequiresCaptcha};
use crate::utils::users::check_email_status;
use crate::utils::users::verify_pasword;
use crate::App;

pub async fn login(
    app: App,
    _requires_captcha: RequiresCaptcha,
    LocalInstanceSettings(settings): LocalInstanceSettings,
    Json(form): Json<LoginUser>,
) -> Result<Response, ApiError> {
    // 1 second / login request
    login_inner(app, form, settings)
        .subtle_timing(Duration::from_secs(1))
        .await
}

#[tracing::instrument(skip(app), name = "v1.users.login")]
async fn login_inner(
    app: App,
    form: LoginUser,
    settings: InstanceSettings,
) -> Result<Response, ApiError> {
    let mut conn = app.db_read().await?;
    let Some(user) = User::find_by_login(&mut conn, &form.name_or_email).await? else {
        return Err(ApiError::new(ApiErrorCategory::LoginUserFailed(
            LoginUserFailed::InvalidCredientials,
        )));
    };

    let password_hash = user.password_hash.to_string();
    let password_matched = spawn_blocking(move || verify_pasword(&form.password, &password_hash))
        .await
        .erase_context()??;

    if !password_matched {
        return Err(ApiError::new(ApiErrorCategory::LoginUserFailed(
            LoginUserFailed::InvalidCredientials,
        )));
    }

    check_email_status(&user, &settings)?;

    Ok(Json(json!({
        "message": "Done!"
    }))
    .into_response())
}
