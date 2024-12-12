use axum::response::{IntoResponse, Response};
use capwat_api_types::routes::instance::{InstanceSettingsResponse, PostSettings, UserSettings};
use capwat_error::ApiError;

use crate::extract::{Json, LocalInstanceSettings, SessionUser};
use crate::services;

pub async fn get_settings(
    user: SessionUser,
    local_settings: LocalInstanceSettings,
) -> Result<Response, ApiError> {
    let response = services::admin::local_instance::GetSettings
        .perform(user, local_settings)
        .await?;

    let response = InstanceSettingsResponse {
        posts: PostSettings {
            max_characters: response.post_max_characters as u16,
        },
        users: UserSettings {
            requires_email_registration: response.require_email_registration,
            requires_email_verification: response.require_email_verification,
        },
    };

    Ok(Json(response).into_response())
}
