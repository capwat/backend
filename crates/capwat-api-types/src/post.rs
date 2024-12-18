use serde::{Deserialize, Serialize};

use crate::user::UserView;
use crate::util::Timestamp;

/// This object contains the summarized data of a post.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PostView {
    pub id: i64,
    pub created_at: Timestamp,
    pub last_edited_at: Option<Timestamp>,
    pub author: Option<UserView>,
    // We'll replicate from Reddit but we're also going to add deleted
    // post data to retain the comments
    pub data: PostData,
}

/// This data represents post content that may be encrypted or unencrypted.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PostData {
    Deleted,
    Public { content: String },
    Encrypted {},
}

crate::should_impl_primitive_traits!(PostView);
crate::should_impl_primitive_traits!(PostData);
