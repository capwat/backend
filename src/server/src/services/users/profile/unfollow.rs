use capwat_error::ext::ResultExt;
use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::id::UserId;
use capwat_model::user::Follower;
use capwat_model::User;
use capwat_utils::Sensitive;

use crate::extract::SessionUser;
use crate::App;

#[derive(Debug)]
pub struct UnfollowUser {
    pub target: Sensitive<UserId>,
}

impl UnfollowUser {
    #[tracing::instrument(name = "services.users.profile.unfollow")]
    pub async fn perform(self, app: &App, session_user: &SessionUser) -> Result<(), ApiError> {
        // Check whether that user exists :)
        let mut conn = app.db_write().await?;

        // The target user must not be themselves
        if session_user.id == *self.target.value() {
            return Err(ApiError::new(ApiErrorCategory::InvalidRequest)
                .message("You cannot unfollow yourself"));
        }

        let Some(target) = User::find(&mut conn, *self.target.value()).await? else {
            let error =
                ApiError::new(ApiErrorCategory::NotFound).message("Could not find user specified");

            return Err(error)?;
        };

        Follower::unfollow(&mut conn, session_user.id, target.id).await?;
        conn.commit().await.erase_context()?;

        Ok(())
    }
}
