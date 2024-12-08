use bon::Builder;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[cfg(feature = "with_diesel")]
use diesel_derive_enum::DbEnum;

use crate::{id::InstanceId, KeyRotationFrequency};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "with_diesel", derive(diesel::Queryable, diesel::Selectable))]
#[cfg_attr(feature = "with_diesel", diesel(table_name = crate::diesel::schema::instance_settings))]
pub struct InstanceSettings {
    pub id: InstanceId,
    pub created: NaiveDateTime,
    pub default_key_rotation_frequency: KeyRotationFrequency,
    pub registration_mode: RegistrationMode,
    pub require_email_registration: bool,
    pub require_email_verification: bool,
    pub require_captcha: bool,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Builder)]
#[cfg_attr(feature = "with_diesel", derive(diesel::AsChangeset))]
#[cfg_attr(feature = "with_diesel", diesel(table_name = crate::diesel::schema::instance_settings))]
pub struct UpdateInstanceSettings {
    pub default_key_rotation_frequency: Option<KeyRotationFrequency>,
    pub registration_mode: Option<RegistrationMode>,
    pub require_email_registration: Option<bool>,
    pub require_email_verification: Option<bool>,
    pub require_captcha: Option<bool>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "with_diesel", derive(DbEnum))]
#[cfg_attr(
    feature = "with_diesel",
    ExistingTypePath = "crate::diesel::schema::sql_types::RegistrationMode"
)]
#[cfg_attr(feature = "with_diesel", DbValueStyle = "kebab-case")]
pub enum RegistrationMode {
    /// Open to all users
    #[default]
    Open,
    /// Open to all users but it requires approval from the
    /// instance administrators.
    RequireApproval,
    /// Closed to public.
    Closed,
}
