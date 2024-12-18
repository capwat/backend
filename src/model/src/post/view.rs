use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};

use super::Post;
use crate::id::UserId;
use crate::user::{User, UserAggregates};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostView {
    pub author: Option<User>,
    pub author_aggregates: Option<UserAggregates>,
    pub post: Post,
}

impl<'r> FromRow<'r, PgRow> for PostView {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let author_id = row.try_get::<Option<UserId>, _>("u.id")?;
        let author = author_id
            .map(|id| {
                Ok::<_, sqlx::Error>(User {
                    id,
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
                })
            })
            .transpose()?;

        let author_aggregates = author_id
            .map(|id| {
                Ok::<_, sqlx::Error>(UserAggregates {
                    id,
                    updated: row.try_get("ua.updated")?,
                    following: row.try_get("ua.following")?,
                    followers: row.try_get("ua.followers")?,
                    posts: row.try_get("ua.posts")?,
                })
            })
            .transpose()?;

        Ok(Self {
            author,
            author_aggregates,
            post: Post {
                id: row.try_get("p.id")?,
                created: row.try_get("p.created")?,
                author_id: row.try_get("p.author_id")?,
                content: row.try_get("p.content")?,
                updated: row.try_get("p.updated")?,
            },
        })
    }
}
