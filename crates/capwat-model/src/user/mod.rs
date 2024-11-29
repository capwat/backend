use bon::Builder;
use chrono::NaiveDateTime;

use crate::id::UserId;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "with_diesel", derive(diesel::Queryable, diesel::Selectable))]
#[cfg_attr(feature = "with_diesel", diesel(table_name = crate::diesel::schema::users))]
pub struct User {
    pub id: UserId,
    pub created: NaiveDateTime,
    pub updated: Option<NaiveDateTime>,
    pub name: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub password_hash: String,
}

#[derive(Builder)]
pub struct InsertUser<'a> {
    #[builder(into)]
    pub name: &'a str,
    pub display_name: Option<&'a str>,
    pub email: Option<&'a str>,
    #[builder(into)]
    pub password_hash: &'a str,
}

#[derive(Builder)]
#[cfg_attr(feature = "with_diesel", derive(diesel::AsChangeset))]
#[cfg_attr(feature = "with_diesel", diesel(table_name = crate::diesel::schema::users))]
pub struct UpdateUser<'a> {
    #[builder(into)]
    pub id: UserId,
    #[builder(into)]
    pub name: Option<&'a str>,
    pub display_name: Option<Option<&'a str>>,
    #[builder(into)]
    pub email: Option<&'a str>,
    pub email_verified: Option<bool>,
    #[builder(into)]
    pub password_hash: Option<&'a str>,
}
