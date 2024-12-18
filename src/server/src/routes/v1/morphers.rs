use capwat_api_types::post::{PostData, PostView as ApiPostView};
use capwat_api_types::user::{UserProfile as ApiUserProfile, UserView as ApiUserView};
use capwat_model::post::PostView;
use capwat_model::setup_user_flags;
use capwat_model::user::UserView;

use crate::extract::SessionUser;

pub trait IntoApiPostView {
    fn into_api_post_view(self) -> ApiPostView;
}

pub trait IntoApiUserProfile {
    fn into_api_user_profile(self) -> ApiUserProfile;
}

pub trait IntoApiUserView {
    fn into_api_user_view(self) -> ApiUserView;
}

impl IntoApiUserProfile for UserView {
    #[must_use]
    fn into_api_user_profile(self) -> ApiUserProfile {
        ApiUserProfile {
            id: self.user.id.0,
            joined_at: self.user.created.into(),
            name: self.user.name,
            flags: self.flags,
            display_name: self.user.display_name,
            // TODO: Find ways to avoid getting an unexpected value when reached negatives?
            followers: self.aggregates.followers as u64,
            following: self.aggregates.following as u64,
        }
    }
}

impl IntoApiUserProfile for SessionUser {
    fn into_api_user_profile(self) -> ApiUserProfile {
        UserView {
            aggregates: self.aggregates,
            flags: self.flags,
            user: self.user,
        }
        .into_api_user_profile()
    }
}

impl IntoApiUserView for SessionUser {
    fn into_api_user_view(self) -> ApiUserView {
        UserView {
            aggregates: self.aggregates,
            flags: self.flags,
            user: self.user,
        }
        .into_api_user_view()
    }
}

impl IntoApiUserView for UserView {
    #[must_use]
    fn into_api_user_view(self) -> ApiUserView {
        ApiUserView {
            id: self.user.id.0,
            joined_at: self.user.created.into(),
            name: self.user.name,
            flags: self.flags,
            display_name: self.user.display_name,
            // TODO: Find ways to avoid getting an unexpected value when reached negatives?
            followers: self.aggregates.followers as u64,
            following: self.aggregates.following as u64,
        }
    }
}

impl IntoApiPostView for PostView {
    #[must_use]
    fn into_api_post_view(self) -> ApiPostView {
        ApiPostView {
            id: self.post.id.0,
            created_at: self.post.created.into(),
            last_edited_at: self.post.updated.map(|v| v.into()),
            author: self
                .author
                .zip(self.author_aggregates)
                .map(|(user, aggregates)| ApiUserView {
                    flags: setup_user_flags(&user, &aggregates),
                    id: user.id.0,
                    joined_at: user.created.into(),
                    name: user.name,
                    display_name: user.display_name,
                    // TODO: Find ways to avoid getting an unexpected value when reached negatives?
                    followers: aggregates.followers as u64,
                    following: aggregates.following as u64,
                }),
            data: match self.post.content {
                Some(content) => PostData::Public { content },
                None => PostData::Deleted,
            },
        }
    }
}
