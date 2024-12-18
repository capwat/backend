use crate::util::Timestamp;
use serde::{Deserialize, Serialize};

pub mod salt;
pub use self::salt::*;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct UserView {
    pub id: i64,
    pub joined_at: Timestamp,
    pub name: String,
    pub display_name: Option<String>,
    pub is_admin: bool,
    pub followers: u64,
    pub following: u64,
}
