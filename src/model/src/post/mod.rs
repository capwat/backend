use bon::Builder;
use capwat_macros::SeaTable;
use chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::id::{PostId, UserId};

mod view;
pub use self::view::*;

#[derive(Debug, Clone, PartialEq, Eq, FromRow, SeaTable)]
#[sea_table(table_name = "posts")]
pub struct Post {
    pub id: PostId,
    pub created: NaiveDateTime,
    pub author_id: Option<UserId>,
    pub content: Option<String>,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Builder)]
pub struct InsertPost<'a> {
    pub author_id: UserId,
    pub content: &'a str,
}

#[derive(Builder)]
pub struct EditPost<'a> {
    pub id: PostId,
    pub new_content: &'a str,
}
