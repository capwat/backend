use sqlx::{postgres::PgRow, FromRow, Row};

use super::{User, UserAggregates};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserView {
    pub aggregates: UserAggregates,
    pub user: User,
}

impl<'r> FromRow<'r, PgRow> for UserView {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            aggregates: UserAggregates {
                id: row.try_get("a.id")?,
                updated: row.try_get("a.updated")?,
                following: row.try_get("a.following")?,
                followers: row.try_get("a.followers")?,
                posts: row.try_get("a.posts")?,
            },
            user: User {
                id: row.try_get("u.id")?,
                created: row.try_get("u.created")?,
                name: row.try_get("u.name")?,
                admin: row.try_get("u.admin")?,
                display_name: row.try_get("u.display_name")?,
                email: row.try_get("u.email")?,
                email_verified: row.try_get("u.email_verified")?,
                access_key_hash: row.try_get("u.access_key_hash")?,
                encrypted_symmetric_key: row.try_get("u.encrypted_symmetric_key")?,
                salt: row.try_get("u.salt")?,
                updated: row.try_get("u.updated")?,
            },
        })
    }
}
