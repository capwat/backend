use bon::Builder;
use chrono::NaiveDateTime;
use diesel::{AsChangeset, Queryable, Selectable};

use crate::id::UserId;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::postgres::schema::users)]
pub struct User {
    pub id: UserId,
    pub created: NaiveDateTime,
    pub name: String,

    pub admin: bool,
    pub display_name: Option<String>,

    pub email: Option<String>,
    pub email_verified: bool,

    pub access_key_hash: String,
    pub encrypted_symmetric_key: String,

    pub salt: String,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Builder)]
pub struct InsertUser<'a> {
    pub name: &'a str,
    pub display_name: Option<&'a str>,
    pub email: Option<&'a str>,
    pub access_key_hash: &'a str,
    pub encrypted_symmetric_key: &'a str,
    pub salt: &'a str,
}

#[derive(Builder, AsChangeset)]
#[diesel(table_name = crate::postgres::schema::users)]
pub struct UpdateUser<'a> {
    #[builder(into)]
    pub id: UserId,
    #[builder(into)]
    pub name: Option<&'a str>,
    pub admin: Option<bool>,
    pub display_name: Option<Option<&'a str>>,
    #[builder(into)]
    pub email: Option<&'a str>,
    pub email_verified: Option<bool>,
}
