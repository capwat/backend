use base64::{prelude::BASE64_URL_SAFE, Engine};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct EncodedBase64(Vec<u8>);

impl EncodedBase64 {
    /// Creates an [`EncodedBase64`] object from raw data.
    #[must_use]
    pub fn from_bytes(data: impl AsRef<[u8]>) -> Self {
        Self(data.as_ref().to_vec())
    }

    #[cfg(feature = "server")]
    #[must_use]
    pub fn from_encoded(encoded: &str) -> Option<Self> {
        BASE64_URL_SAFE.decode(encoded).ok().map(Self)
    }

    /// Returns the decoded array of bytes from the encoded
    /// Base64 string.
    #[must_use]
    pub fn decode(&self) -> &[u8] {
        &self.0
    }

    /// Encodes it back to Base64.
    #[must_use]
    pub fn encode(&self) -> String {
        BASE64_URL_SAFE.encode(&self.0)
    }
}

impl<T: AsRef<[u8]>> From<T> for EncodedBase64 {
    fn from(value: T) -> Self {
        Self::from_bytes(value)
    }
}

impl Debug for EncodedBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EncodedBase64(...)")
    }
}

impl Display for EncodedBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.encode(), f)
    }
}

impl<'de> Deserialize<'de> for EncodedBase64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = EncodedBase64;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("base64 encoded string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                BASE64_URL_SAFE
                    .decode(v)
                    .map(EncodedBase64)
                    .map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl Serialize for EncodedBase64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}
