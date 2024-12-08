use bon::Builder;
use chrono::NaiveDateTime;

use crate::id::UserId;
use crate::key::KeyRotationFrequency;

mod keys;
pub use self::keys::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "with_diesel", derive(diesel::Queryable, diesel::Selectable))]
#[cfg_attr(feature = "with_diesel", diesel(table_name = crate::diesel::schema::users))]
pub struct User {
    pub id: UserId,
    pub created: NaiveDateTime,
    pub name: String,

    pub admin: bool,
    pub display_name: Option<String>,
    pub key_rotation_frequency: KeyRotationFrequency,

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

    pub public_key: &'a str,
    pub encrypted_secret_key: &'a str,
    pub key_rotation_frequency: Option<KeyRotationFrequency>,
}

#[derive(Builder)]
#[cfg_attr(feature = "with_diesel", derive(diesel::AsChangeset))]
#[cfg_attr(feature = "with_diesel", diesel(table_name = crate::diesel::schema::users))]
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
