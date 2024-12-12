use diesel::dsl::exists;
use thiserror::Error;

use crate::id::UserId;
use crate::user::Follower;

use super::prelude::*;
use super::schema::followers;

#[derive(Debug, Error)]
#[error("Could not follow source user to target user")]
pub struct FollowError;

#[derive(Debug, Error)]
#[error("Could not unfollow source user to target user")]
pub struct UnfollowError;

impl Follower {
    #[tracing::instrument(skip_all, name = "db.followers.unfollow")]
    pub async fn unfollow(
        conn: &mut PgConnection<'_>,
        source_id: UserId,
        target_id: UserId,
    ) -> Result<bool, UnfollowError> {
        let filter = followers::source_id
            .eq(source_id)
            .and(followers::target_id.eq(target_id));

        let exists = diesel::delete(followers::table.filter(filter))
            .get_result::<Self>(&mut *conn)
            .await
            .optional()
            .change_context(UnfollowError)?
            .is_some();

        Ok(exists)
    }

    #[tracing::instrument(skip_all, name = "db.followers.follow")]
    pub async fn follow(
        conn: &mut PgConnection<'_>,
        source_id: UserId,
        target_id: UserId,
    ) -> Result<(), FollowError> {
        // TODO: Reduce into one operation only as much as I can.
        let filter = followers::source_id
            .eq(source_id)
            .and(followers::target_id.eq(target_id));

        let exists_before = diesel::select(exists(followers::table.filter(filter)))
            .get_result::<bool>(&mut *conn)
            .await
            .change_context(FollowError)
            .attach_printable("could not check if source user has followed already")?;

        if exists_before {
            return Ok(());
        }

        diesel::insert_into(followers::table)
            .values((
                followers::source_id.eq(source_id),
                followers::target_id.eq(target_id),
            ))
            .execute(&mut *conn)
            .await
            .change_context(FollowError)
            .attach_printable("could not insert follower data")?;

        Ok(())
    }

    #[tracing::instrument(skip_all, name = "db.followers.get")]
    pub async fn get(
        conn: &mut PgConnection<'_>,
        source_id: UserId,
        target_id: UserId,
    ) -> Result<Option<Self>> {
        let filter = followers::source_id
            .eq(source_id)
            .and(followers::target_id.eq(target_id));

        followers::table
            .filter(filter)
            .get_result::<Self>(&mut *conn)
            .await
            .optional()
            .erase_context()
            .attach_printable("could not find follower data by source or target id")
    }
}
