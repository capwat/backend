use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::post::PostView;

use crate::extract::SessionUser;
use crate::App;

/// Gets user's newest post feed from their followers sorted from
/// newest to oldest filtered what are posted last 2 days ago or how
/// long they're been offline.
pub struct GetPostFeed {
    pub page: Option<u64>,

    // Our default limit is 20 posts/request but we do accept
    // requests up to 35 posts/request only.
    pub limit: Option<u64>,
}

impl GetPostFeed {
    const MIN_LIMIT: u64 = 5;
    const MAX_LIMIT: u64 = 35;
    const DEFAULT_LIMIT: u64 = 20;

    #[tracing::instrument(skip_all, fields(self), name = "services.posts.feed")]
    pub async fn perform(
        self,
        app: &App,
        session_user: &SessionUser,
    ) -> Result<Vec<PostView>, ApiError> {
        let limit = self.limit.unwrap_or(Self::DEFAULT_LIMIT);
        let page = self.page.unwrap_or(0);

        // there must be at least 5 to 35 posts/request only
        if !(Self::MIN_LIMIT..=Self::MAX_LIMIT).contains(&limit) {
            return Err(ApiError::new(ApiErrorCategory::InvalidRequest).message("Invalid limit!"));
        }

        let mut conn = app.db_read().await?;
        let posts = PostView::list_for_user_feed(&mut conn, session_user.id, page, limit).await?;

        Ok(posts)
    }
}
