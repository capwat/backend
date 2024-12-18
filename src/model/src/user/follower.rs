use capwat_macros::SeaTable;
use chrono::NaiveDateTime;
use sqlx::FromRow;

use super::UserView;
use crate::id::{FollowerId, UserId};

#[derive(Debug, Clone, FromRow, SeaTable)]
#[sea_table(table_name = "followers")]
pub struct Follower {
    pub id: FollowerId,
    pub created: NaiveDateTime,
    pub source_id: UserId,
    pub target_id: UserId,
}

#[derive(Debug, Clone)]
pub struct FollowerView {
    pub id: FollowerId,
    pub followed_at: NaiveDateTime,
    pub target: UserView,
}
