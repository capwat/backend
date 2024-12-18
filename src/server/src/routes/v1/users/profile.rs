pub mod me {
    use crate::extract::{Json, LocalInstanceSettings, SessionUser};
    use crate::routes::v1::morphers::{IntoApiPostView, IntoApiUserProfile, IntoApiUserView};
    use crate::{services, App};

    use axum::extract::Query;
    use axum::response::{IntoResponse, Response};
    use axum::Router;
    use capwat_api_types::routes::users::{
        CurrentUserFollowerEntry, ListCurrentUserFollowers, ListCurrentUserPosts, PublishPost,
        PublishPostResponse,
    };
    use capwat_error::ApiError;
    use capwat_model::id::PostId;
    use capwat_utils::Sensitive;

    pub fn routes() -> Router<App> {
        use axum::routing::{get, post};

        Router::new()
            .route("/", get(my_profile))
            .route("/followers", get(followers))
            .route("/posts", get(posts))
            .route("/posts", post(publish_post))
    }

    pub async fn followers(
        app: App,
        session_user: SessionUser,
        Query(query): Query<ListCurrentUserFollowers>,
    ) -> Result<Response, ApiError> {
        let request = services::users::profile::GetLocalProfileFollowers {
            page: query.page,
            limit: query.limit,
            order: query.order,
        };

        let response = request
            .perform(&app, &session_user)
            .await?
            .into_iter()
            .map(|view| CurrentUserFollowerEntry {
                followed_at: view.followed_at.into(),
                user: view.target.into_api_user_view(),
            })
            .collect::<Vec<_>>();

        Ok(Json(response).into_response())
    }

    pub async fn my_profile(session_user: SessionUser) -> Result<Response, ApiError> {
        let view = services::users::profile::LocalProfile
            .perform(session_user)
            .await
            .session_user;

        let response = Json(view.into_api_user_profile());
        Ok(response.into_response())
    }

    pub async fn posts(
        app: App,
        session_user: SessionUser,
        Query(query): Query<ListCurrentUserPosts>,
    ) -> Result<Response, ApiError> {
        let request = services::users::posts::GetLocalProfilePosts {
            before: query.before.map(PostId),
            limit: query.limit,
        };

        let response = request
            .perform(&app, &session_user)
            .await?
            .into_iter()
            .map(|view| view.into_api_post_view())
            .collect::<Vec<_>>();

        Ok(Json(response).into_response())
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
        });

        Ok(response.into_response())
    }
}

pub mod others {
    use crate::extract::SessionUser;
    use crate::{services, App};

    use axum::extract::Path;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    use axum::Router;
    use capwat_error::ApiError;
    use capwat_model::id::UserId;

    pub fn routes() -> Router<App> {
        use axum::routing::post;

        Router::new()
            .route("/follow", post(follow))
            .route("/unfollow", post(unfollow))
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::test_utils;

        use axum_test::TestServer;
        use serde_json::json;

        mod me {
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

        mod follow {
            use super::*;
            use capwat_model::user::Follower;

            #[capwat_macros::api_test]
            async fn should_work(app: App, mut server: TestServer) {
                let alice = test_utils::users::override_credentials()
                    .app(&app)
                    .server(&mut server)
                    .name("alice")
                    .call()
                    .await;

                let bob = test_utils::users::register()
                    .name("bob")
                    .app(&app)
                    .call()
                    .await;

                let response = server
                    .post(&format!("/api/v1/users/{}/follow", bob.user_id))
                    .await;

                response.assert_status_ok();

                // checking if they really follow someone
                let mut conn = app.db_read().await.unwrap();
                let data = Follower::get(&mut conn, alice.user.id, bob.user_id)
                    .await
                    .unwrap();

                assert!(data.is_some());
            }
        }
    }
}
