mod follow;
mod unfollow;

pub mod current_user;

pub use self::follow::FollowUser;
pub use self::unfollow::UnfollowUser;

use crate::extract::SessionUser;
use crate::App;

use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::id::UserId;
use capwat_model::user::UserView;
use capwat_utils::Sensitive;

pub enum GetProfile<'a> {
    Username(Sensitive<&'a str>),
    Id(Sensitive<UserId>),
}

impl<'a> std::fmt::Debug for GetProfile<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Username(name) => f
                .debug_struct("GetProfile")
                .field("search_type", &"username")
                .field("specifier", &name)
                .finish(),

            Self::Id(id) => f
                .debug_struct("GetProfile")
                .field("search_type", &"user_id")
                .field("specifier", &id)
                .finish(),
        }
    }
}

impl GetProfile<'_> {
    #[tracing::instrument(skip_all, name = "services.users.profile")]
    pub async fn perform(
        self,
        app: &App,
        _session_user: &Option<SessionUser>,
    ) -> Result<UserView, ApiError> {
        let mut conn = app.db_read().await?;
        let result = match self {
            Self::Username(entry) => UserView::find_by_username(&mut conn, &entry).await,
            Self::Id(entry) => UserView::find(&mut conn, *entry).await,
        }?;

        if let Some(entry) = result {
            Ok(entry)
        } else {
            Err(ApiError::new(ApiErrorCategory::NotFound))
        }
    }
}
