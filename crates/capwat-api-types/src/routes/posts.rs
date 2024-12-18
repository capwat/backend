use serde::{Deserialize, Serialize};

use crate::util::{Pagination, Timestamp};

/// Get a list of posts published from the user's followers.
///
/// **ROUTE**: `GET /posts/feed?limit=<limit>&offset=<offset>`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct GetPostFeed {
    #[serde(default, flatten)]
    pub pagination: Pagination,
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
    pub created_at: Timestamp,
}

/// Edit a post from a specific ID.
///
/// **ROUTE**: `PATCH /user/@me/posts/{}`
#[derive(Debug)]
pub struct EditPost {
    pub content: String,
}
