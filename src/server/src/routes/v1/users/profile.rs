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

#[cfg(test)]
mod tests {
    use crate::test_utils;

    #[tracing::instrument]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_follow_user() {
        let (mut server, app, _) = test_utils::build_test_server().await;
        test_utils::users::start_server_session()
            .app(&app)
            .server(&mut server)
            .name("alice")
            .call()
            .await;

        let bob = test_utils::users::register()
            .app(&app)
            .name("bob")
            .call()
            .await;

        let response = server
            .post(&format!("/api/v1/users/{}/follow", bob.user_id.0))
            .await;

        response.assert_status_ok();
    }
}
