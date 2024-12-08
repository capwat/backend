use chrono::{Duration, NaiveDateTime};
#[cfg(feature = "with_diesel")]
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "with_diesel", derive(DbEnum))]
#[cfg_attr(
    feature = "with_diesel",
    ExistingTypePath = "crate::diesel::schema::sql_types::KeyRotationFrequency"
)]
#[cfg_attr(feature = "with_diesel", DbValueStyle = "kebab-case")]
pub enum KeyRotationFrequency {
    #[default]
    Monthly,
    Weekly,
}

impl KeyRotationFrequency {
    /// Generates expiry timestamp based on the timestamp and its
    /// [variant](KeyRotationFrequency) given.
    pub fn get_expiry_timestamp(self, timestamp: NaiveDateTime) -> NaiveDateTime {
        match self {
            // We'll take 1 month = 4 weeks.
            Self::Monthly => timestamp + Duration::weeks(4),
            Self::Weekly => timestamp + Duration::weeks(1),
        }
    }
}
