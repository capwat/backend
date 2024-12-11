use axum::response::{IntoResponse, Response};
use capwat_api_types::routes::users::{
    LocalUserProfile, LoginUser, LoginUserResponse, RegisterUser, RegisterUserResponse,
};
use capwat_error::ApiError;
use capwat_utils::Sensitive;

use crate::extract::{Json, LocalInstanceSettings, SessionUser};
use crate::{services, App};

pub async fn local_profile(user: SessionUser) -> Result<Response, ApiError> {
    let user = services::users::LocalProfile
        .perform(user)
        .await
        .user
        .into_inner();

    let response = Json(LocalUserProfile {
        id: user.id.0,
        name: user.name,
        display_name: user.display_name,
    });

    Ok(response.into_response())
}

pub async fn login(
    app: App,
    local_settings: LocalInstanceSettings,
    Json(form): Json<LoginUser>,
) -> Result<Response, ApiError> {
    let request = services::users::Login {
        name_or_email: Sensitive::new(&form.name_or_email),
        access_key_hash: form.access_key_hash.as_ref().map(|v| Sensitive::new(v)),
    };

    let response = request.perform(&app, &local_settings).await?;
    let response = Json(LoginUserResponse {
        name: response.user.name,
        display_name: response.user.display_name,
        email_verified: local_settings
            .require_email_verification
            .then(|| response.user.email_verified),
        encrypted_symmetric_key: response.user.encrypted_symmetric_key,
        token: response.token,
    });

    Ok(response.into_response())
}

pub async fn register(
    app: App,
    local_settings: LocalInstanceSettings,
    Json(form): Json<RegisterUser>,
) -> Result<Response, ApiError> {
    let request = services::users::Register {
        name: Sensitive::new(&form.name),
        email: form.email.as_deref().map(Sensitive::new),
        access_key_hash: Sensitive::new(&form.access_key_hash),
        salt: Sensitive::new(&form.salt),
        symmetric_key: Sensitive::new(&form.symmetric_key),
    };

    request.perform(&app, &local_settings).await?;
    let response = Json(RegisterUserResponse {
        verify_email: local_settings.require_email_verification,
    });

    Ok(response.into_response())
}

#[cfg(test)]
mod tests {
    use crate::util::test as util;

    mod login {
        use super::*;
        use capwat_api_types::routes::users::LoginUser;

        #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn should_login_user() {
            let (server, app, _) = util::build_test_server().await;
            let user = util::init_test_user().app(&app).name("alice").call().await;

            let request = LoginUser::builder()
                .name_or_email("alice")
                .access_key_hash(user.params.access_key_hash)
                .build();

            let response = server.post("/api/v1/users/login").json(&request).await;
            response.assert_status_ok();
        }
    }
}
