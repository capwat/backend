use serde::{Deserialize, Serialize};

/// Publishes a post.
///
/// **ROUTE**: `POST /posts`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PublishPost {
    pub content: String,
}

/// A response after `POST /posts` has successfully performed.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CreatePostResponse {
    pub id: i64,
}

/// Edit a post from a specific ID.
///
/// **ROUTE**: `PATCH /posts/{}`
#[derive(Debug)]
pub struct EditPost {
    pub content: String,
}
