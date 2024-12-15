use axum::response::{IntoResponse, Response};
use capwat_api_types::routes::users::{
    LoginUser, LoginUserResponse, RegisterUser, RegisterUserResponse,
};
use capwat_error::ApiError;
use capwat_utils::Sensitive;

use crate::extract::{Json, LocalInstanceSettings};
use crate::{services, App};

pub mod profile;

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
        id: response.user.id.0,
        name: response.user.name,
        joined_at: response.user.created.into(),
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
    use super::*;
    use crate::test_utils;

    use axum_test::TestServer;
    use serde_json::json;

    mod local_profile {
        use super::*;

        #[capwat_macros::api_test]
        async fn should_get_their_profile(app: App, mut server: TestServer) {
            let alice = test_utils::users::override_credentials()
                .app(&app)
                .server(&mut server)
                .name("alice")
                .call()
                .await;

            let response = server.get("/api/v1/users/@me").await;
            response.assert_status_ok();
            response.assert_json_contains(&json!({
                "id": alice.user.id,
                "name": alice.user.name,
                "display_name": alice.user.display_name,

                "followers": 0,
                "following": 0,
                "posts": 0,
            }));
        }

        #[capwat_macros::api_test]
        async fn should_restrict_if_no_auth_is_presented(server: TestServer) {
            let response = server.get("/api/v1/users/@me").await;
            response.assert_status_unauthorized();
            response.assert_json_contains(&json!({ "code": "access_denied" }));
        }
    }

    mod login {
        use super::*;
        use capwat_api_types::routes::users::LoginUser;

        #[capwat_macros::api_test]
        async fn should_login_user(app: App, server: TestServer) {
            let credentials = test_utils::users::register()
                .app(&app)
                .name("alice")
                .call()
                .await;

            let request = LoginUser::builder()
                .name_or_email("alice")
                .access_key_hash(credentials.access_key_hash.clone())
                .build();

            let response = server.post("/api/v1/users/login").json(&request).await;
            response.assert_status_ok();
            response.assert_json_contains(&json!({
                "name": "alice",
                "display_name": None::<String>,
                "encrypted_symmetric_key": credentials.encrypted_symmetric_key.encode(),
            }));
        }
    }

    mod register {
        use super::*;
        use capwat_api_types::routes::users::RegisterUser;
        use capwat_model::instance::UpdateInstanceSettings;

        #[capwat_macros::api_test]
        async fn should_register_user(server: TestServer) {
            let params = test_utils::generate_register_user_params("alice").await;

            let request = RegisterUser::builder()
                .name("alice".into())
                .salt(params.salt)
                .access_key_hash(params.access_key_hash)
                .symmetric_key(params.encrypted_symmetric_key)
                .build();

            let response = server.post("/api/v1/users/register").json(&request).await;
            response.assert_status_ok();
            response.assert_json_contains(&json!({
                "verify_email": false,
            }));
        }

        #[capwat_macros::api_test]
        async fn should_set_verify_email_to_true_if_required(app: App, server: TestServer) {
            let params = test_utils::generate_register_user_params("alice").await;

            let mut conn = app.db_write().await.unwrap();
            UpdateInstanceSettings::builder()
                .require_email_verification(true)
                .build()
                .perform_local(&mut conn)
                .await
                .unwrap();

            conn.commit().await.unwrap();

            let request = RegisterUser::builder()
                .name("alice".into())
                .salt(params.salt)
                .access_key_hash(params.access_key_hash)
                .symmetric_key(params.encrypted_symmetric_key)
                .build();

            let response = server.post("/api/v1/users/register").json(&request).await;
            response.assert_status_ok();
            response.assert_json_contains(&json!({
                "verify_email": true,
            }));
        }
    }
}
