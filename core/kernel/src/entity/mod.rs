pub use capwat_types::id;
pub use capwat_types::Timestamp;

use self::id::{Id, UserMarker};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: Id<UserMarker>,
    pub created_at: Timestamp,
    pub name: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub password_hash: String,
    pub updated_at: Option<Timestamp>,
}
