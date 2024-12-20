use serde::{Deserialize, Serialize};

use crate::user::UserView;
use crate::util::{SortOrder, Timestamp};

/// Get a list of posts posted from the current user.
///
/// This object must be used as query parameters to perform
/// this request.
///
/// **ROUTE**: `GET /users/@me/posts`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ListCurrentUserPosts {
    /// The highest post ID in the previous page.
    pub before: Option<i64>,
    /// Maximum number of posts to fetch (1-15)
    pub limit: Option<u64>,
}

/// Publishes a post.
///
/// **ROUTE**: `POST /user/@me/posts`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PublishPost {
    pub content: String,
}

/// A response after `POST /user/@me/posts` has successfully performed.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PublishPostResponse {
    pub id: i64,
}

/// Edit a post from a specific ID.
///
/// **ROUTE**: `PATCH /user/@me/posts/{}`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct EditCurrentUserPost {
    pub content: String,
}

/// Get a list of users who got followed to the current user.
///
/// This object must be used as query parameters to perform
/// this request.
///
/// **ROUTE**: `GET /users/@me/followers`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ListCurrentUserFollowers {
    pub page: Option<u64>,
    /// Maximum number of posts to fetch (1-50)
    pub limit: Option<u64>,
    pub order: Option<SortOrder>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CurrentUserFollowerEntry {
    pub followed_at: Timestamp,
    #[serde(flatten)]
    pub user: UserView,
}

/// Get a list of users who got followed by the current user.
///
/// This object must be used as query parameters to perform
/// this request.
///
/// **ROUTE**: `GET /users/@me/followers`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ListCurrentUserFollowing {
    /// The highest post ID in the previous page.
    pub after: Option<i64>,
    /// Maximum number of posts to fetch (1-50)
    pub limit: Option<u64>,
}

crate::should_impl_primitive_traits!(ListCurrentUserPosts);
crate::should_impl_primitive_traits!(PublishPost);
crate::should_impl_primitive_traits!(EditCurrentUserPost);

crate::should_impl_primitive_traits!(ListCurrentUserFollowers);
crate::should_impl_primitive_traits!(ListCurrentUserFollowing);
