use bon::Builder;
use chrono::NaiveDateTime;
use diesel::{Queryable, Selectable};

use crate::id::{PostId, UserId};

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::postgres::schema::posts)]
pub struct Post {
    pub id: PostId,
    pub created: NaiveDateTime,
    pub author_id: UserId,
    pub content: String,
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
