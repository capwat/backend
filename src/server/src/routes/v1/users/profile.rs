use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use capwat_error::ApiError;
use capwat_model::id::UserId;

use crate::extract::{Json, LocalInstanceSettings, SessionUser};
use crate::{services, App};

pub mod local {
    use super::*;
    use capwat_api_types::routes::posts::{PublishPost, PublishPostResponse};
    use capwat_api_types::routes::users::LocalUserProfile;
    use capwat_utils::Sensitive;

    pub async fn view(session_user: SessionUser) -> Result<Response, ApiError> {
        let user_view = services::users::profile::LocalProfile
            .perform(session_user)
            .await
            .session_user;

        let response = Json(LocalUserProfile {
            id: user_view.id.0,
            joined_at: user_view.created.into(),
            name: user_view.user.name,
            display_name: user_view.user.display_name,

            // TODO: Find a future-proof way to mitigate past the i64 limit
            followers: user_view.aggregates.followers as u64,
            following: user_view.aggregates.following as u64,
            posts: user_view.aggregates.posts as u64,
        });

        Ok(response.into_response())
    }

    pub async fn publish_post(
        app: App,
        session_user: SessionUser,
        local_settings: LocalInstanceSettings,
        Json(form): Json<PublishPost>,
    ) -> Result<Response, ApiError> {
        let request = services::users::posts::PublishUserPost {
            content: Sensitive::new(&form.content),
        };

        let response = request
            .perform(&app, &local_settings, &session_user)
            .await?;

        let response = Json(PublishPostResponse {
            id: response.post.id.0,
            created_at: response.post.created.into(),
        });

        Ok(response.into_response())
    }
}

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

pub async fn unfollow(
    app: App,
    user: SessionUser,
    Path(target_id): Path<UserId>,
) -> Result<Response, ApiError> {
    let request = services::users::profile::UnfollowUser {
        target: target_id.into(),
    };
    request.perform(&app, &user).await?;

    Ok(StatusCode::OK.into_response())
}
