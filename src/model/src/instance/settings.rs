use bon::Builder;
use capwat_macros::SeaTable;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

use crate::id::InstanceId;

#[derive(Debug, Builder, Clone, FromRow, SeaTable)]
#[sea_table(changeset = "UpdateInstanceSettings", table_name = "instance_settings")]
pub struct InstanceSettings {
    #[builder(default = InstanceId(0))]
    pub id: InstanceId,
    #[builder(default = chrono::Utc::now().naive_utc())]
    pub created: NaiveDateTime,
    #[builder(default = 200)]
    pub post_max_characters: i32,
    #[builder(default)]
    #[sea_table(exclude_in_changeset)]
    pub registration_mode: RegistrationMode,
    #[builder(default = false)]
    pub require_email_registration: bool,
    #[builder(default = false)]
    pub require_email_verification: bool,
    #[builder(default = false)]
    pub require_captcha: bool,
    pub updated: Option<NaiveDateTime>,
}

#[derive(Builder)]
pub struct UpdateInstanceSettings {
    pub post_max_characters: Option<i32>,
    pub registration_mode: Option<RegistrationMode>,
    pub require_email_registration: Option<bool>,
    pub require_email_verification: Option<bool>,
    pub require_captcha: Option<bool>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, Type)]
#[serde(rename_all = "kebab-case")]
#[sqlx(rename_all = "kebab-case", type_name = "registration_mode")]
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
