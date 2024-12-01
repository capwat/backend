use base64::{prelude::BASE64_URL_SAFE, Engine};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

const ML_KEM768_PUB_KEY_LEN: usize = 1184;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PostQuantumKey {
    MlKem768([u8; ML_KEM768_PUB_KEY_LEN]),
    #[cfg(not(feature = "server"))]
    Unsupported {
        code: u8,
        data: Vec<u8>,
    },
}

impl PostQuantumKey {
    #[must_use]
    pub fn algorithm(&self) -> PostQuantumKeyType {
        match self {
            Self::MlKem768(..) => PostQuantumKeyType::MlKem768,
            #[cfg(not(feature = "server"))]
            Self::Unsupported { code, .. } => PostQuantumKeyType::Unsupported(*code),
        }
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::MlKem768(data) => data,
            #[cfg(not(feature = "server"))]
            Self::Unsupported { data, .. } => &data,
        }
    }
}

impl Display for PostQuantumKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let algorithm = self.algorithm();
        let data = self.as_bytes();

        let mut buffer = Vec::with_capacity(1 + data.len());
        buffer.push(algorithm.value());
        buffer.extend_from_slice(data);

        Display::fmt(&BASE64_URL_SAFE.encode(&buffer), f)
    }
}

impl<'de> Deserialize<'de> for PostQuantumKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = PostQuantumKey;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("capwat post-quantum key")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let decoded = BASE64_URL_SAFE
                    .decode(v)
                    .map_err(serde::de::Error::custom)?;

                if decoded.len() < 1 {
                    return Err(serde::de::Error::custom("invalid post-quantum key value"));
                }

                let algorithm = PostQuantumKeyType::from_value(decoded[0]);
                let data = decoded[1..].to_vec();

                #[cfg(feature = "server")]
                let Some(algorithm) = algorithm
                else {
                    return Err(serde::de::Error::custom(format!(
                        "unknown post quantum key algorithm: {}",
                        decoded[0]
                    )));
                };

                #[cfg(not(feature = "server"))]
                let algorithm = algorithm.unwrap();

                Ok(match algorithm {
                    PostQuantumKeyType::MlKem768 => {
                        PostQuantumKey::MlKem768(data.try_into().map_err(|_| {
                            serde::de::Error::custom("invalid length for ML-KEM768")
                        })?)
                    }
                    #[cfg(not(feature = "server"))]
                    PostQuantumKeyType::Unsupported(code) => {
                        PostQuantumKey::Unsupported { code, data }
                    }
                })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl Serialize for PostQuantumKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

/// A list of classic cryptographic types supported by Capwat
/// at the moment in this version of this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostQuantumKeyType {
    MlKem768,
    #[cfg(not(feature = "server"))]
    /// Unsupported algorithm. Maybe this algorithm is not implemented
    /// during the time of this crate.
    Unsupported(u8),
}

impl PostQuantumKeyType {
    #[must_use]
    pub fn from_value(value: u8) -> Option<Self> {
        match value {
            0x1 => Some(Self::MlKem768),
            #[cfg(feature = "server")]
            _ => None,
            #[cfg(not(feature = "server"))]
            _ => Self::Unsupported(value),
        }
    }

    #[must_use]
    pub fn value(&self) -> u8 {
        match self {
            Self::MlKem768 => 0x1,
            #[cfg(not(feature = "server"))]
            Self::Unsupported(n) => *n,
        }
    }
}
