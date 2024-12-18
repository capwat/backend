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
#[error("could not get follower view by {0}")]
pub struct FindFollowerViewError(&'static str);

#[derive(Debug, Error)]
#[error("could not get a list of followers")]
pub struct GetFollowerListError;

impl FollowerView {
    #[tracing::instrument(skip_all, name = "db.post_view.find")]
    pub async fn get_list(
        conn: &mut PgConnection,
        source_id: UserId,
        page: u64,
        limit: u64,
    ) -> Result<Vec<Self>, GetFollowerListError> {
        // to avoid any possible SQL issues there
        let limit = limit.min(5);
        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(FollowerIdent::Followers)
            .and_where(Expr::col(FollowerIdent::SourceId).eq(source_id.0))
            .order_by(FollowerIdent::Created, Order::Desc)
            .offset(page * limit)
            .limit(limit)
            .build_sqlx(PostgresQueryBuilder);

        let mut raw_followers = sqlx::query_as_with::<_, Follower, _>(&sql, values)
            .fetch_all(&mut *conn)
            .await
            .change_context(GetFollowerListError)?;

        let mut list = Vec::<Self>::with_capacity(raw_followers.len());
        for follower in raw_followers.drain(..) {
            let user = UserView::find(conn, follower.target_id)
                .await
                .change_context(GetFollowerListError)
                .attach_printable("could not get user view data for the target user")?
                .ok_or_else(|| {
                    Error::unknown(GetFollowerListError)
                        .attach_printable("unexpected target user is missing")
                })?;

            list.push(Self {
                id: follower.id,
                followed_at: follower.created,
                user,
            });
        }

        Ok(list)
    }

    #[tracing::instrument(skip_all, name = "db.post_view.find")]
    pub async fn find_by_target(
        conn: &mut PgConnection,
        source_id: UserId,
        target_id: UserId,
    ) -> Result<Option<Self>, FindFollowerViewError> {
        const ERROR_CTX: FindFollowerViewError =
            FindFollowerViewError("target user id from source user id");

        // We'll going to use multiple queries to do this because I'm too
        // lazy to optimize this into one query only.
        //
        // TODO: Try to optimize multiple queries into a single query only
        let Some(follower) = Follower::get(conn, source_id, target_id)
            .await
            .change_context(ERROR_CTX)
            .attach_printable("could not get follower data")?
        else {
            return Ok(None);
        };

        let user = UserView::find(conn, target_id)
            .await
            .change_context(ERROR_CTX)
            .attach_printable("could not get user view from target user")?
            .ok_or_else(|| {
                Error::unknown(ERROR_CTX).attach_printable("unexpected target user is missing")
            })?;

        Ok(Some(Self {
            id: follower.id,
            followed_at: follower.created,
            user,
        }))
    }
}
