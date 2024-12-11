use bon::Builder;
use chrono::NaiveDateTime;
use diesel::{AsChangeset, Queryable, Selectable};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

use crate::id::InstanceId;

#[derive(Debug, Builder, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::postgres::schema::instance_settings)]
pub struct InstanceSettings {
    #[builder(default = InstanceId(0))]
    pub id: InstanceId,
    #[builder(default = chrono::Utc::now().naive_utc())]
    pub created: NaiveDateTime,
    #[builder(default)]
    pub registration_mode: RegistrationMode,
    #[builder(default = false)]
    pub require_email_registration: bool,
    #[builder(default = false)]
    pub require_email_verification: bool,
    #[builder(default = false)]
    pub require_captcha: bool,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Builder, AsChangeset)]
#[diesel(table_name = crate::postgres::schema::instance_settings)]
pub struct UpdateInstanceSettings {
    pub registration_mode: Option<RegistrationMode>,
    pub require_email_registration: Option<bool>,
    pub require_email_verification: Option<bool>,
    pub require_captcha: Option<bool>,
}

#[derive(Debug, DbEnum, Default, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[ExistingTypePath = "crate::postgres::schema::sql_types::RegistrationMode"]
#[DbValueStyle = "kebab-case"]
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
