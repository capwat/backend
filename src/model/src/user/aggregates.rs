use capwat_macros::SeaTable;
use chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::id::UserId;

#[derive(Debug, Clone, FromRow, PartialEq, Eq, SeaTable)]
#[sea_table(table_name = "user_aggregates")]
pub struct UserAggregates {
    pub id: UserId,
    pub updated: NaiveDateTime,

    pub following: i64,
    pub followers: i64,
    pub posts: i64,
}
