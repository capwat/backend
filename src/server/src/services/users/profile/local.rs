use capwat_api_types::util::SortOrder;
use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::user::FollowerView;

use crate::extract::SessionUser;
use crate::App;

pub struct LocalProfile;

impl LocalProfile {
    #[tracing::instrument(skip_all, name = "services.users.profile.me")]
    pub async fn perform(self, session_user: SessionUser) -> LocalProfileResponse {
        LocalProfileResponse { session_user }
    }
}

#[must_use]
pub struct LocalProfileResponse {
    pub session_user: SessionUser,
}

#[derive(Debug)]
pub struct GetLocalProfileFollowers {
    pub page: Option<u64>,
    pub order: Option<SortOrder>,

    // Our default limit is up to 30 followers/request
    pub limit: Option<u64>,
}

impl GetLocalProfileFollowers {
    const MIN_LIMIT: u64 = 5;
    const MAX_LIMIT: u64 = 30;
    const DEFAULT_LIMIT: u64 = 30;

    #[tracing::instrument(skip_all, fields(self), name = "services.users.profile.me.followers")]
    pub async fn perform(
        self,
        app: &App,
        session_user: &SessionUser,
    ) -> Result<Vec<FollowerView>, ApiError> {
        let limit = self.limit.unwrap_or(Self::DEFAULT_LIMIT);

        // there must be at least 5 to 35 posts/request only
        if !(Self::MIN_LIMIT..=Self::MAX_LIMIT).contains(&limit) {
            return Err(ApiError::new(ApiErrorCategory::InvalidRequest).message("Invalid limit!"));
        }

        let mut conn = app.db_read().await?;
        let list = FollowerView::list_from_current_user(
            &mut conn,
            session_user.id,
            limit,
            self.page,
            self.order,
        )
        .await?;

        Ok(list)
    }
}
