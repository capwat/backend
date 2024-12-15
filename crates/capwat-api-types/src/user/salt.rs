use base64::Engine;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use thiserror::Error;

const USER_SALT_SIZE: usize = 16;

/// It contains user's salt with an an array of 16 bytes to help
/// to generate secured password hashes, encryption keys and others.
///
///
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserSalt([u8; USER_SALT_SIZE]);

impl UserSalt {
    pub const SIZE: usize = USER_SALT_SIZE;

    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl Debug for UserSalt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UserSalt({})", hex::encode(self.0))
    }
}

impl Display for UserSalt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use base64::prelude::BASE64_URL_SAFE;
        Display::fmt(&BASE64_URL_SAFE.encode(self.0), f)
    }
}

impl AsRef<[u8]> for UserSalt {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<[u8; USER_SALT_SIZE]> for UserSalt {
    fn from(value: [u8; USER_SALT_SIZE]) -> Self {
        Self(value)
    }
}

#[derive(Debug, Error)]
pub enum InvalidUserSalt {
    #[error(transparent)]
    Encoding(base64::DecodeError),
    #[error("User salts must be an equal size of {} bytes", USER_SALT_SIZE)]
    InvalidLength(usize),
}

impl FromStr for UserSalt {
    type Err = InvalidUserSalt;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use base64::prelude::BASE64_URL_SAFE;

        let raw_bytes = BASE64_URL_SAFE
            .decode(s)
            .map_err(InvalidUserSalt::Encoding)?;

        if raw_bytes.len() != USER_SALT_SIZE {
            Err(InvalidUserSalt::InvalidLength(raw_bytes.len()))?;
        }

        Ok(Self(raw_bytes.try_into().unwrap()))
    }
}

impl<'de> serde::Deserialize<'de> for UserSalt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = UserSalt;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("Capwat user salt")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                UserSalt::from_str(v).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl serde::Serialize for UserSalt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}
