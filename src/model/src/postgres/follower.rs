use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::Result;
use sea_query::{Asterisk, Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::PgConnection;
use thiserror::Error;

use crate::id::UserId;
use crate::user::{Follower, FollowerIdent};

#[derive(Debug, Error)]
#[error("Could not follow source user to target user")]
pub struct FollowError;

#[derive(Debug, Error)]
#[error("Could not unfollow source user to target user")]
pub struct UnfollowError;

impl Follower {
    #[tracing::instrument(skip_all, name = "db.followers.unfollow")]
    pub async fn unfollow(
        conn: &mut PgConnection,
        source_id: UserId,
        target_id: UserId,
    ) -> Result<(), UnfollowError> {
        let (sql, values) = Query::delete()
            .from_table(FollowerIdent::Followers)
            .and_where(
                Expr::col(FollowerIdent::SourceId)
                    .eq(source_id.0)
                    .and(Expr::col(FollowerIdent::TargetId).eq(target_id.0)),
            )
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, Self, _>(&sql, values)
            .fetch_optional(conn)
            .await
            .change_context(UnfollowError)?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.followers.follow")]
    pub async fn follow(
        conn: &mut PgConnection,
        source_id: UserId,
        target_id: UserId,
    ) -> Result<(), FollowError> {
        let (sql, values) = Query::select()
            .expr(Expr::exists(
                Query::select()
                    .column(Asterisk)
                    .from(FollowerIdent::Followers)
                    .and_where(
                        Expr::col(FollowerIdent::SourceId)
                            .eq(source_id.0)
                            .and(Expr::col(FollowerIdent::TargetId).eq(target_id.0)),
                    )
                    .take(),
            ))
            .build_sqlx(PostgresQueryBuilder);

        let exists_before = sqlx::query_scalar_with::<_, bool, _>(&sql, values)
            .fetch_one(&mut *conn)
            .await
            .change_context(FollowError)
            .attach_printable(
                "could not check if source user has followed the target user already",
            )?;

        if exists_before {
            return Ok(());
        }

        let (sql, values) = Query::insert()
            .into_table(FollowerIdent::Followers)
            .columns([FollowerIdent::SourceId, FollowerIdent::TargetId])
            .values_panic([source_id.0.into(), target_id.0.into()])
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with::<_, _>(&sql, values)
            .execute(conn)
            .await
            .change_context(FollowError)
            .attach_printable("could not insert follower data")?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.followers.get")]
    pub async fn get(
        conn: &mut PgConnection,
        source_id: UserId,
        target_id: UserId,
    ) -> Result<Option<Self>> {
        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(FollowerIdent::Followers)
            .and_where(
                Expr::col(FollowerIdent::SourceId)
                    .eq(source_id.0)
                    .and(Expr::col(FollowerIdent::TargetId).eq(target_id.0)),
            )
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, Self, _>(&sql, values)
            .fetch_optional(conn)
            .await
            .erase_context()
            .attach_printable("could not find follower data by source or target id")
    }
}
