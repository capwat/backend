mod publish;
pub use self::publish::*;

use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::id::PostId;
use capwat_model::post::PostView;

use crate::extract::SessionUser;
use crate::App;

pub struct GetLocalProfilePosts {
    pub before: Option<PostId>,

    // Our default limit is 20 posts/request but we do accept
    // requests up to 35 posts/request only.
    pub limit: Option<u64>,
}

impl GetLocalProfilePosts {
    const MIN_LIMIT: u64 = 5;
    const MAX_LIMIT: u64 = 35;
    const DEFAULT_LIMIT: u64 = 20;

    #[tracing::instrument(skip_all, fields(self), name = "services.users.me.get_posts")]
    pub async fn perform(
        self,
        app: &App,
        session_user: &SessionUser,
    ) -> Result<Vec<PostView>, ApiError> {
        let limit = self.limit.unwrap_or(Self::DEFAULT_LIMIT);

        // there must be at least 5 to 35 posts/request only
        if !(Self::MIN_LIMIT..=Self::MAX_LIMIT).contains(&limit) {
            return Err(ApiError::new(ApiErrorCategory::InvalidRequest).message("Invalid limit!"));
        }

        let mut conn = app.db_read().await?;
        let posts =
            PostView::list_from_current_user(&mut conn, session_user.id, self.before, limit)
                .await?;

        Ok(posts)
    }
}
