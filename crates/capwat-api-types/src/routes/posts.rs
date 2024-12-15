use serde::{Deserialize, Serialize};

use crate::util::Timestamp;

/// Publishes a post.
///
/// **ROUTE**: `POST /posts`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PublishPost {
    pub content: String,
}

/// A response after `POST /posts` has successfully performed.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PublishPostResponse {
    pub id: i64,
    pub created_at: Timestamp,
}

/// Edit a post from a specific ID.
///
/// **ROUTE**: `PATCH /posts/{}`
#[derive(Debug)]
pub struct EditPost {
    pub content: String,
}
