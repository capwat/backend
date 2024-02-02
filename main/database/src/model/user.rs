use capwat_kernel::entity::id::{Id, UserMarker};
use capwat_kernel::entity::Timestamp;

use diesel::deserialize::Queryable;

#[derive(Debug, Clone, PartialEq, Eq, Queryable)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: Id<UserMarker>,
    pub created_at: Timestamp,
    pub name: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub password_hash: String,
    pub updated_at: Option<Timestamp>,
}

impl From<capwat_kernel::entity::User> for User {
    fn from(value: capwat_kernel::entity::User) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at.into(),
            name: value.name,
            email: value.email,
            display_name: value.display_name,
            password_hash: value.password_hash,
            updated_at: value.updated_at.map(|v| v.into()),
        }
    }
}

impl From<User> for capwat_kernel::entity::User {
    fn from(value: User) -> Self {
        use capwat_kernel::entity::User as KernelUser;
        KernelUser {
            id: value.id,
            created_at: value.created_at.into(),
            name: value.name,
            email: value.email,
            display_name: value.display_name,
            password_hash: value.password_hash,
            updated_at: value.updated_at.map(|v| v.into()),
        }
    }
}
