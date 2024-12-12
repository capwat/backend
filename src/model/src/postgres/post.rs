use capwat_db::pool::PgConnection;
use capwat_error::Result;
use thiserror::Error;

use crate::id::PostId;
use crate::post::{InsertPost, Post};

use super::prelude::*;
use super::schema::posts;

impl Post {
    #[tracing::instrument(skip_all, name = "db.posts.find")]
    pub async fn find(conn: &mut PgConnection<'_>, id: PostId) -> Result<Self> {
        posts::table
            .filter(posts::id.eq(id))
            .get_result::<Self>(&mut *conn)
            .await
            .erase_context()
            .attach_printable("could not find post by id")
    }
}

#[derive(Debug, Error)]
#[error("Could not insert post")]
pub struct InsertPostError;

impl InsertPost<'_> {
    #[tracing::instrument(skip_all, name = "db.posts.insert")]
    pub async fn insert(&self, conn: &mut PgConnection<'_>) -> Result<Post, InsertPostError> {
        diesel::insert_into(posts::table)
            .values((
                posts::author_id.eq(self.author_id),
                posts::content.eq(self.content),
            ))
            .get_result::<Post>(&mut *conn)
            .await
            .change_context(InsertPostError)
    }
}
