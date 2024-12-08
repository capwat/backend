use axum::response::{IntoResponse, Response};
use capwat_api_types::routes::instance::{InstanceSettingsResponse, UserSettings};
use capwat_error::ApiError;

use crate::auth::Identity;
use crate::extract::{Json, LocalInstanceSettings};

#[tracing::instrument(name = "v1.admin.instance.settings")]
pub async fn get(
    identity: Identity,
    LocalInstanceSettings(settings): LocalInstanceSettings,
) -> Result<Response, ApiError> {
    identity.requires_admin()?;

    let response = InstanceSettingsResponse {
        users: UserSettings {
            requires_email_registration: settings.require_email_registration,
            requires_email_verification: settings.require_email_verification,
        },
    };

    Ok(Json(response).into_response())
}
