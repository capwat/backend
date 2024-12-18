use serde::{Deserialize, Serialize};

use crate::util::Timestamp;

/// A response after `GET /admin/instance/settings` has successfully performed.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct InstanceSettingsResponse {
    pub last_updated: Timestamp,
    pub posts: PostSettings,
    pub users: UserSettings,
}

/// This part of settings is related to posts management.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PostSettings {
    pub max_characters: u16,
}

/// This part of settings is related to user management.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct UserSettings {
    pub requires_email_registration: bool,
    pub requires_email_verification: bool,
}

crate::should_impl_primitive_traits!(InstanceSettingsResponse);
crate::should_impl_primitive_traits!(PostSettings);
crate::should_impl_primitive_traits!(UserSettings);
