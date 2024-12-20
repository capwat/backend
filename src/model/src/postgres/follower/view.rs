use capwat_api_types::util::SortOrder;
use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::{Error, Result};
use sea_query::{Asterisk, Expr, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::PgConnection;
use thiserror::Error;

use crate::id::UserId;
use crate::user::{Follower, FollowerView, UserView};

use super::FollowerIdent;

#[derive(Debug, Error)]
#[error("could not find follower view from {0}")]
pub struct FindFollowerViewError(&'static str);

impl FollowerView {
    #[tracing::instrument(skip_all, name = "db.follower_view.list_from_current_user")]
    pub async fn list_from_current_user(
        conn: &mut PgConnection,
        current_user_id: UserId,
        limit: u64,
        page: Option<u64>,
        order: Option<SortOrder>,
    ) -> Result<Vec<Self>> {
        let order = order.unwrap_or_default();

        let mut stmt = Query::select();
        stmt.column(Asterisk)
            .from(FollowerIdent::Followers)
            .and_where(Expr::col(FollowerIdent::TargetId).eq(current_user_id.0));

        match order {
            SortOrder::Descending => {
                stmt.order_by(FollowerIdent::Created, Order::Desc);
            }
            SortOrder::Ascending => {
                stmt.order_by(FollowerIdent::Created, Order::Asc);
            }
        }

        let (sql, values) = stmt
            .offset(page.unwrap_or(0) * limit)
            .limit(limit)
            .build_sqlx(PostgresQueryBuilder);

        let followers = sqlx::query_as_with::<_, Follower, _>(&sql, values)
            .fetch_all(&mut *conn)
            .await
            .erase_context()
            .attach_printable("could not fetch list of followers from the current user")?;

        let mut result = Vec::with_capacity(followers.len());
        for follower in followers {
            let source = UserView::find(&mut *conn, follower.source_id)
                .await
                .and_then(|v| v.ok_or_else(|| Error::unknown_generic(sqlx::Error::RowNotFound)))
                .attach_printable(
                    "could not get target user's data to fetch list of followers from the current user",
                )?;

            result.push(Self {
                id: follower.id,
                followed_at: follower.created,
                source,
            });
        }

        Ok(result)
    }
}

impl FollowerView {}

#[cfg(test)]
mod tests {
    use capwat_db::PgPooledConnection;
    use capwat_error::Result;

    use crate::postgres::users::tests::{generate_alice, generate_user};
    use crate::user::{Follower, FollowerView};

    #[capwat_macros::postgres_query_test]
    async fn should_fetch_list_from_current_user(mut conn: PgPooledConnection) -> Result<()> {
        let (alice, _) = generate_alice(&mut conn).await?;
        let (bob, _) = generate_user(&mut conn, "bob").await?;
        let (carol, _) = generate_user(&mut conn, "carol").await?;
        let (darren, _) = generate_user(&mut conn, "darren").await?;
        let (earl, _) = generate_user(&mut conn, "earl").await?;
        let (fred, _) = generate_user(&mut conn, "fred").await?;

        Follower::follow(&mut conn, earl.id, alice.id).await?;
        Follower::follow(&mut conn, bob.id, alice.id).await?;
        Follower::follow(&mut conn, darren.id, alice.id).await?;
        Follower::follow(&mut conn, carol.id, alice.id).await?;
        Follower::follow(&mut conn, fred.id, alice.id).await?;

        let mut list = FollowerView::list_from_current_user(&mut conn, alice.id, 2, None, None)
            .await?
            .into_iter()
            .map(|v| v.source.user.id);

        assert_eq!(Some(fred.id), list.next());
        assert_eq!(Some(carol.id), list.next());
        assert_eq!(None, list.next());

        let mut list = FollowerView::list_from_current_user(&mut conn, alice.id, 2, Some(1), None)
            .await?
            .into_iter()
            .map(|v| v.source.user.id);

        assert_eq!(Some(darren.id), list.next());
        assert_eq!(Some(bob.id), list.next());
        assert_eq!(None, list.next());

        Ok(())
    }
}
