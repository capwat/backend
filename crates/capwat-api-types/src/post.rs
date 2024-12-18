use serde::{Deserialize, Serialize};

use crate::user::UserView;
use crate::util::Timestamp;

/// This object contains the summarized data of a post.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PostView {
    pub id: i64,
    pub created_at: Timestamp,
    pub last_edited_at: Timestamp,

    pub author: UserView,
    pub content: PostContent,
}

/// This data represents post content that may be encrypted or unencrypted.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum PostContent {
    Public { data: String },
    Encrypted {},
}

crate::should_impl_primitive_traits!(PostView);
crate::should_impl_primitive_traits!(PostContent);
