use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::Result;
use sea_query::{Asterisk, Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::PgConnection;
use thiserror::Error;

use crate::id::PostId;
use crate::post::{InsertPost, Post, PostIdent};

mod view;

impl Post {
    #[tracing::instrument(skip_all, name = "db.posts.find")]
    pub async fn find(conn: &mut PgConnection, id: PostId) -> Result<Option<Self>> {
        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(PostIdent::Posts)
            .and_where(Expr::col(PostIdent::Id).eq(id.0))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, Self, _>(&sql, values)
            .fetch_optional(conn)
            .await
            .erase_context()
            .attach_printable("could not find post by id")
    }

    /// Deletes the post content but not the author.
    #[must_use]
    #[tracing::instrument(skip_all, name = "db.posts.remove")]
    pub async fn delete(conn: &mut PgConnection, id: PostId) -> Result<()> {
        let (sql, values) = Query::update()
            .table(PostIdent::Posts)
            .and_where(Expr::col(PostIdent::Id).eq(id.0))
            .value(PostIdent::Content, None::<String>)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, Self, _>(&sql, values)
            .fetch_optional(conn)
            .await
            .erase_context()
            .attach_printable("could not remove post")?;

        Ok(())
    }
}

#[derive(Debug, Error)]
#[error("Could not insert post")]
pub struct InsertPostError;

impl InsertPost<'_> {
    #[tracing::instrument(skip_all, name = "db.posts.insert")]
    pub async fn insert(&self, conn: &mut PgConnection) -> Result<Post, InsertPostError> {
        let (sql, values) = Query::insert()
            .into_table(PostIdent::Posts)
            .columns([PostIdent::AuthorId, PostIdent::Content])
            .values_panic([self.author_id.0.into(), self.content.into()])
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, Post, _>(&sql, values)
            .fetch_one(conn)
            .await
            .change_context(InsertPostError)
    }
}
