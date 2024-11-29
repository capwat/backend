use diesel_derive_newtype::DieselNewType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "with_diesel", derive(DieselNewType))]
pub struct UserId(pub i64);

impl From<i64> for UserId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

#[derive(DieselNewType, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceId(pub i32);

impl From<i32> for InstanceId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
