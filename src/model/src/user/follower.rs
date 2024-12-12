use chrono::NaiveDateTime;
use diesel::{Queryable, Selectable};

use crate::id::{FollowerId, UserId};

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::postgres::schema::followers)]
pub struct Follower {
    pub id: FollowerId,
    pub created: NaiveDateTime,
    pub source_id: UserId,
    pub target_id: UserId,
}
