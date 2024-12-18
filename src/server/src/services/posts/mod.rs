use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::id::PostId;
use capwat_model::post::PostView;

use crate::extract::SessionUser;
use crate::App;

/// Gets user's newest post feed from their followers sorted from
/// newest to oldest filtered what are posted last 2 days ago or how
/// long they're been offline.
pub struct ListPostRecommendations {
    pub before: Option<i64>,

    // Our default limit is 15 posts/request but we do accept
    // requests up to 25 posts/request only.
    pub limit: Option<u64>,
}

impl ListPostRecommendations {
    const MIN_LIMIT: u64 = 10;
    const MAX_LIMIT: u64 = 25;
    const DEFAULT_LIMIT: u64 = 15;

    #[tracing::instrument(skip_all, fields(self), name = "services.posts.feed")]
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
        let posts = PostView::list_for_recommendations(
            &mut conn,
            session_user.id,
            self.before.map(PostId),
            limit,
        )
        .await?;

        Ok(posts)
    }
}
