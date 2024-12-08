use bon::Builder;
use chrono::NaiveDateTime;

use crate::{
    id::{UserId, UserKeysId},
    KeyRotationFrequency,
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "with_diesel", derive(diesel::Queryable, diesel::Selectable))]
#[cfg_attr(feature = "with_diesel", diesel(table_name = crate::diesel::schema::user_keys))]
pub struct UserKeys {
    pub id: UserKeysId,
    pub user_id: UserId,
    pub created: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub public_key: String,
    pub encrypted_secret_key: String,
}

#[derive(Builder)]
pub struct InsertUserKeys<'a> {
    pub user_id: UserId,
    pub rotation_frequency: KeyRotationFrequency,
    pub public_key: &'a str,
    pub encrypted_secret_key: &'a str,
}
