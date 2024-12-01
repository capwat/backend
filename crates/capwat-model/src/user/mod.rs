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

    pub access_key_hash: String,
    pub root_classic_pk: String,
    pub root_encrypted_classic_sk: String,
    pub root_pqc_pk: String,
    pub root_encrypted_pqc_sk: String,
}

#[derive(Builder)]
pub struct InsertUser<'a> {
    pub name: &'a str,
    pub display_name: Option<&'a str>,
    pub email: Option<&'a str>,
    pub access_key_hash: &'a str,
    pub root_classic_pk: &'a str,
    pub root_encrypted_classic_sk: &'a str,
    pub root_pqc_pk: &'a str,
    pub root_encrypted_pqc_sk: &'a str,
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
}
