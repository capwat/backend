use crate::util::Timestamp;
use serde::{Deserialize, Serialize};

pub mod salt;
pub use self::salt::*;

/// This object represents user's profile.
///
/// This type of schema will be received after `/users/:id` or `/users/@me`
/// has fetched successfully.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct UserProfile {
    pub id: i64,
    pub joined_at: Timestamp,
    pub name: String,
    pub display_name: Option<String>,
    pub is_admin: bool,

    pub followers: u64,
    pub following: u64,
}

crate::should_impl_primitive_traits!(UserProfile);

/// This object represents the summarized data of a user.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct UserView {
    pub id: i64,
    pub joined_at: Timestamp,
    pub name: String,
    pub display_name: Option<String>,
    pub is_admin: bool,
}

crate::should_impl_primitive_traits!(UserView);
crate::should_impl_primitive_traits!(UserSalt);
