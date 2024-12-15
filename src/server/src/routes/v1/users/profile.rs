use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use capwat_error::ApiError;
use capwat_model::id::UserId;

use crate::extract::SessionUser;
use crate::{services, App};

pub async fn follow(
    app: App,
    user: SessionUser,
    Path(target_id): Path<UserId>,
) -> Result<Response, ApiError> {
    let request = services::users::profile::FollowUser {
        target: target_id.into(),
    };
    request.perform(&app, &user).await?;

    Ok(StatusCode::OK.into_response())
}
