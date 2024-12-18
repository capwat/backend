use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::Result;
use sea_query::{
    Expr, Iden, IntoColumnRef, IntoIden, Order, PostgresQueryBuilder, Query, SelectStatement,
    TableRef,
};
use sea_query_binder::SqlxBinder;
use sqlx::PgConnection;

use crate::id::{PostId, UserId};
use crate::post::{Post, PostView};
use crate::postgres::into_view_aliases;
use crate::user::{FollowerIdent, User, UserAggregates, UserAggregatesIdent, UserIdent};

use super::PostIdent;

#[derive(Debug, Clone, Iden)]
enum LocalIdent {
    /// Alias for `follower`
    F,
    /// Alias for `users`
    U,
    /// Alias for `posts`
    P,
    /// Alias for `user_aggregates`
    UA,
}

impl PostView {
    #[tracing::instrument(skip_all, name = "db.post_view.find")]
    pub async fn find(conn: &mut PgConnection, id: PostId) -> Result<Option<Self>> {
        let (sql, values) = Self::generate_select_stmt()
            .and_where(Expr::col((LocalIdent::P, PostIdent::Id)).eq(id.0))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, Self, _>(&sql, values)
            .fetch_optional(conn)
            .await
            .erase_context()
            .attach_printable("could not find post view from post id")
    }

    #[tracing::instrument(skip_all, name = "db.posts.list_from_current_user")]
    pub async fn list_from_current_user(
        conn: &mut PgConnection,
        current_user_id: UserId,
        before: Option<PostId>,
        limit: u64,
    ) -> Result<Vec<Self>> {
        let mut stmt = Self::generate_select_stmt();
        stmt.and_where(Expr::col((LocalIdent::P, PostIdent::AuthorId)).eq(current_user_id.0))
            .order_by_columns([
                ((LocalIdent::P, PostIdent::Created), Order::Desc),
                ((LocalIdent::P, PostIdent::Id), Order::Desc),
            ])
            .limit(limit);

        if let Some(after) = before {
            stmt.and_where(Expr::col((LocalIdent::P, PostIdent::Id)).lt(after.0));
        }

        let (sql, values) = stmt.build_sqlx(PostgresQueryBuilder);
        sqlx::query_as_with::<_, Self, _>(&sql, values)
            .fetch_all(conn)
            .await
            .erase_context()
            .attach_printable("could not fetch list of posts from the current user")
    }

    #[tracing::instrument(skip_all, name = "db.posts.list_for_recommendations")]
    pub async fn list_for_recommendations(
        conn: &mut PgConnection,
        current_user_id: UserId,
        before: Option<PostId>,
        limit: u64,
    ) -> Result<Vec<Self>> {
        let mut stmt = Self::generate_select_stmt();
        stmt.left_join(
            TableRef::Table(FollowerIdent::Followers.into_iden()).alias(LocalIdent::F),
            Expr::col((LocalIdent::F, FollowerIdent::TargetId))
                .eq(Expr::col((LocalIdent::U, UserIdent::Id))),
        )
        .and_where(Expr::col((LocalIdent::F, FollowerIdent::SourceId)).eq(current_user_id.0))
        // it must be not discoverable if it is deleted or the
        // author's account is deleted
        .and_where(Expr::col((LocalIdent::P, PostIdent::Content)).is_not_null())
        .and_where(Expr::col((LocalIdent::P, PostIdent::AuthorId)).is_not_null())
        .order_by_columns([
            ((LocalIdent::P, PostIdent::Created), Order::Desc),
            ((LocalIdent::P, PostIdent::Id), Order::Desc),
        ])
        .limit(limit);

        if let Some(after) = before {
            stmt.and_where(Expr::col((LocalIdent::P, PostIdent::Id)).lt(after.0));
        }

        let (sql, values) = stmt.build_sqlx(PostgresQueryBuilder);
        sqlx::query_as_with::<_, Self, _>(&sql, values)
            .fetch_all(conn)
            .await
            .erase_context()
            .attach_printable("could not fetch list of recommended posts")
    }
}

impl PostView {
    fn generate_select_stmt() -> SelectStatement {
        Query::select()
            .exprs(into_view_aliases(
                User::make_view_columns(LocalIdent::U).into_iter(),
            ))
            .exprs(into_view_aliases(
                Post::make_view_columns(LocalIdent::P).into_iter(),
            ))
            .exprs(into_view_aliases(
                UserAggregates::make_view_columns(LocalIdent::UA).into_iter(),
            ))
            .from_as(PostIdent::Posts, LocalIdent::P)
            .left_join(
                TableRef::Table(UserIdent::Users.into_iden()).alias(LocalIdent::U),
                Expr::col((LocalIdent::U, UserIdent::Id))
                    .eq(Expr::col((LocalIdent::P, PostIdent::AuthorId))),
            )
            .left_join(
                TableRef::Table(UserAggregatesIdent::UserAggregates.into_iden())
                    .alias(LocalIdent::UA),
                Expr::col((LocalIdent::U, UserIdent::Id))
                    .eq(Expr::col((LocalIdent::UA, UserAggregatesIdent::Id))),
            )
            .group_by_columns([
                (LocalIdent::P, PostIdent::Id).into_column_ref(),
                (LocalIdent::U, UserIdent::Id).into_column_ref(),
                (LocalIdent::UA, UserIdent::Id).into_column_ref(),
            ])
            .take()
    }
}

#[cfg(test)]
mod tests {
    use capwat_db::PgPooledConnection;
    use capwat_error::Result;

    use crate::post::{InsertPost, PostView};
    use crate::postgres::users::tests::{generate_alice, generate_user};
    use crate::user::{Follower, UserAggregates};

    #[capwat_macros::postgres_query_test]
    async fn should_generate_list_for_user_feed(mut conn: PgPooledConnection) -> Result<()> {
        let (alice, _) = generate_alice(&mut conn).await?;
        let (bob, _) = generate_user(&mut conn, "bob").await?;
        let (caryl, _) = generate_user(&mut conn, "caryl").await?;

        // Alice follows Bob and Caryl
        Follower::follow(&mut conn, alice.id, bob.id).await?;
        Follower::follow(&mut conn, alice.id, caryl.id).await?;

        // Bob posts something
        let bob_post_1 = InsertPost::builder()
            .author_id(bob.id)
            .content("Hello, World!")
            .build()
            .insert(&mut conn)
            .await?;

        let bob_post_2 = InsertPost::builder()
            .author_id(bob.id)
            .content("Hello, World!")
            .build()
            .insert(&mut conn)
            .await?;

        let caryl_post_1 = InsertPost::builder()
            .author_id(bob.id)
            .content("Hello, World!")
            .build()
            .insert(&mut conn)
            .await?;

        let bob_post_3 = InsertPost::builder()
            .author_id(bob.id)
            .content("Hello, World!")
            .build()
            .insert(&mut conn)
            .await?;

        let caryl_post_2 = InsertPost::builder()
            .author_id(bob.id)
            .content("Hello, World!")
            .build()
            .insert(&mut conn)
            .await?;

        let mut feed = PostView::list_for_recommendations(&mut conn, alice.id, None, 3)
            .await?
            .into_iter()
            .map(|v| v.post.id);

        assert_eq!(feed.next(), Some(caryl_post_2.id));
        assert_eq!(feed.next(), Some(bob_post_3.id));
        assert_eq!(feed.next(), Some(caryl_post_1.id));
        assert_eq!(feed.next(), None);

        // after test
        let mut feed =
            PostView::list_for_recommendations(&mut conn, alice.id, Some(caryl_post_1.id), 3)
                .await?
                .into_iter()
                .map(|v| v.post.id);

        assert_eq!(feed.next(), Some(bob_post_2.id));
        assert_eq!(feed.next(), Some(bob_post_1.id));
        assert_eq!(feed.next(), None);

        Ok(())
    }

    #[capwat_macros::postgres_query_test]
    async fn should_generate_list_for_their_posts(mut conn: PgPooledConnection) -> Result<()> {
        let (alice, _) = generate_alice(&mut conn).await?;

        // Alice posts something
        let post_1 = InsertPost::builder()
            .author_id(alice.id)
            .content("Hello, World!")
            .build()
            .insert(&mut conn)
            .await?;

        let post_2 = InsertPost::builder()
            .author_id(alice.id)
            .content("Hello, World!")
            .build()
            .insert(&mut conn)
            .await?;

        let mut their_posts = PostView::list_from_current_user(&mut conn, alice.id, None, 5)
            .await?
            .into_iter()
            .map(|v| v.post.id);

        assert_eq!(their_posts.next(), Some(post_2.id));
        assert_eq!(their_posts.next(), Some(post_1.id));
        assert_eq!(their_posts.next(), None);

        Ok(())
    }

    #[capwat_macros::postgres_query_test]
    async fn test_post_view(mut conn: PgPooledConnection) -> Result<()> {
        let (alice, _) = generate_alice(&mut conn).await?;

        let primitive = InsertPost::builder()
            .author_id(alice.id)
            .content("Hello, World!")
            .build()
            .insert(&mut conn)
            .await?;

        let aggregates = UserAggregates::find(&mut conn, alice.id).await?.unwrap();
        let view = PostView::find(&mut conn, primitive.id).await?;

        assert_eq!(
            view,
            Some(PostView {
                author: Some(alice),
                author_aggregates: Some(aggregates),
                post: primitive
            })
        );

        Ok(())
    }
}
