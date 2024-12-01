use base64::{prelude::BASE64_URL_SAFE, Engine};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::str::FromStr;
use thiserror::Error;

const USER_SALT_SIZE: usize = 16;

/// It contains user's decoded salt from encoded Base64.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserSalt([u8; USER_SALT_SIZE]);

impl UserSalt {
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    pub fn consume(&self) -> [u8; USER_SALT_SIZE] {
        self.0
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

impl Debug for UserSalt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UserSalt({self})")
    }
}

impl Display for UserSalt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&BASE64_URL_SAFE.encode(&self.0), f)
    }
}

#[derive(Debug, Error)]
pub enum InvalidUserSalt {
    #[error("Failed to decode user salt: {0}")]
    Encoding(base64::DecodeError),
    #[error("Got invalid length for user salt")]
    InvalidLength,
}

impl FromStr for UserSalt {
    type Err = InvalidUserSalt;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = BASE64_URL_SAFE
            .decode(s)
            .map_err(|e| InvalidUserSalt::Encoding(e))?;

        let array: [u8; USER_SALT_SIZE] = decoded
            .try_into()
            .map_err(|_e| InvalidUserSalt::InvalidLength)?;

        Ok(Self(array))
    }
}

impl<'de> Deserialize<'de> for UserSalt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
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

impl Serialize for UserSalt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}
