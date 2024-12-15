use bon::Builder;
use capwat_macros::SeaTable;
use chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::id::UserId;

mod follower;
pub use self::follower::*;

#[derive(Debug, Clone, FromRow, PartialEq, Eq, SeaTable)]
#[sea_table(changeset = "UpdateUser<'_>", table_name = "users")]
pub struct User {
    pub id: UserId,
    pub created: NaiveDateTime,
    pub name: String,

    pub admin: bool,
    pub display_name: Option<String>,

    pub email: Option<String>,
    pub email_verified: bool,

    #[sea_table(exclude_in_changeset)]
    pub access_key_hash: String,
    #[sea_table(exclude_in_changeset)]
    pub encrypted_symmetric_key: String,

    #[sea_table(exclude_in_changeset)]
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

#[derive(Builder)]
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
