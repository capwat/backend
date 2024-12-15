use capwat_error::ext::ResultExt;
use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::id::UserId;
use capwat_model::user::Follower;
use capwat_model::User;
use capwat_utils::Sensitive;

use crate::extract::SessionUser;
use crate::App;

#[derive(Debug)]
pub struct FollowUser {
    pub target: Sensitive<UserId>,
}

impl FollowUser {
    #[tracing::instrument(name = "services.users.profile.follow")]
    pub async fn perform(self, app: &App, session_user: &SessionUser) -> Result<(), ApiError> {
        // Check whether that user exists :)
        let mut conn = app.db_write().await?;

        // The target user must not be themselves
        if session_user.id == *self.target.value() {
            return Err(ApiError::new(ApiErrorCategory::InvalidRequest)
                .message("You cannot follow yourself"));
        }

        let Some(target) = User::find(&mut conn, *self.target.value()).await? else {
            let error =
                ApiError::new(ApiErrorCategory::NotFound).message("Could not find user specified");

            return Err(error)?;
        };

        Follower::follow(&mut conn, session_user.id, target.id).await?;
        conn.commit().await.erase_context()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{self, TestResultExt};

    use assert_json_diff::assert_json_include;
    use serde_json::json;

    #[capwat_macros::api_test]
    async fn should_follow_user(app: App) {
        let alice = test_utils::users::get_session_data()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let bob = test_utils::users::register()
            .app(&app)
            .name("bob")
            .call()
            .await;

        let request = FollowUser {
            target: Sensitive::new(bob.user_id),
        };

        request
            .perform(&app, &alice.get_session_user(&app).await)
            .await
            .unwrap();

        // checking if they really follow someone
        let mut conn = app.db_read().await.unwrap();
        let data = Follower::get(&mut conn, alice.user.id, bob.user_id)
            .await
            .unwrap();

        assert!(data.is_some());
    }

    #[capwat_macros::api_test]
    async fn should_follow_user_if_done_repeatedly(app: App) {
        let alice = test_utils::users::get_session_data()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let bob = test_utils::users::register()
            .app(&app)
            .name("bob")
            .call()
            .await;

        let request = FollowUser {
            target: Sensitive::new(bob.user_id),
        };

        request
            .perform(&app, &alice.get_session_user(&app).await)
            .await
            .unwrap();

        let request = FollowUser {
            target: Sensitive::new(bob.user_id),
        };

        request
            .perform(&app, &alice.get_session_user(&app).await)
            .await
            .unwrap();

        // checking if they really follow someone
        let mut conn = app.db_read().await.unwrap();
        let data = Follower::get(&mut conn, alice.user.id, bob.user_id)
            .await
            .unwrap();

        assert!(data.is_some());
    }

    #[capwat_macros::api_test]
    async fn should_reject_if_target_user_not_found(app: App) {
        let alice = test_utils::users::get_session_data()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let request = FollowUser {
            target: Sensitive::new(UserId(23000000)),
        };

        let error = request
            .perform(&app, &alice.get_session_user(&app).await)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "not_found",
                "message": "Could not find user specified",
            }),
        )
    }

    #[capwat_macros::api_test]
    async fn should_not_follow_themselves(app: App) {
        let alice = test_utils::users::get_session_data()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let request = FollowUser {
            target: Sensitive::new(alice.user.id),
        };

        let error = request
            .perform(&app, &alice.get_session_user(&app).await)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "invalid_request",
                "message": "You cannot follow yourself",
            }),
        )
    }
}
