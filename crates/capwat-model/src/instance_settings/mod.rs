use bon::Builder;
use chrono::NaiveDateTime;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

use crate::id::InstanceId;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "with_diesel", derive(diesel::Queryable, diesel::Selectable))]
#[cfg_attr(feature = "with_diesel", diesel(table_name = crate::diesel::schema::instance_settings))]
pub struct InstanceSettings {
    pub id: InstanceId,
    pub created: NaiveDateTime,
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
    pub registration_mode: Option<RegistrationMode>,
    pub require_email_registration: Option<bool>,
    pub require_email_verification: Option<bool>,
    pub require_captcha: Option<bool>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, DbEnum)]
#[ExistingTypePath = "crate::diesel::schema::sql_types::RegistrationMode"]
#[DbValueStyle = "kebab-case"]
#[serde(rename_all = "kebab-case")]
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
