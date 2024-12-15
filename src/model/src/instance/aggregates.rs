use capwat_macros::SeaTable;
use chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::id::InstanceId;

#[derive(Debug, Clone, FromRow, SeaTable)]
#[sea_table(table_name = "instance_aggregates")]
pub struct InstanceAggregates {
    pub id: InstanceId,
    pub updated: NaiveDateTime,

    pub following: i64,
    pub followers: i64,
    pub posts: i64,
}
