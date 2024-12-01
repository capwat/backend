use base64::{prelude::BASE64_URL_SAFE, Engine};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClassicKey {
    Curve25519([u8; 32]),
    #[cfg(not(feature = "server"))]
    Unsupported {
        code: u8,
        data: Vec<u8>,
    },
}

impl ClassicKey {
    #[must_use]
    pub fn algorithm(&self) -> ClassicKeyType {
        match self {
            Self::Curve25519(..) => ClassicKeyType::Curve25519,
            #[cfg(not(feature = "server"))]
            Self::Unsupported { code, .. } => ClassicKeyType::Unsupported(*code),
        }
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Curve25519(data) => data,
            #[cfg(not(feature = "server"))]
            Self::Unsupported { data, .. } => &data,
        }
    }
}

impl Display for ClassicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let algorithm = self.algorithm();
        let data = self.as_bytes();

        let mut buffer = Vec::with_capacity(1 + data.len());
        buffer.push(algorithm.value());
        buffer.extend_from_slice(data);

        Display::fmt(&BASE64_URL_SAFE.encode(&buffer), f)
    }
}

impl<'de> Deserialize<'de> for ClassicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ClassicKey;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("capwat classic key")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let decoded = BASE64_URL_SAFE
                    .decode(v)
                    .map_err(serde::de::Error::custom)?;

                if decoded.len() < 1 {
                    return Err(serde::de::Error::custom("invalid classic key value"));
                }

                let algorithm = ClassicKeyType::from_value(decoded[0]);
                let data = decoded[1..].to_vec();

                #[cfg(feature = "server")]
                let Some(algorithm) = algorithm
                else {
                    return Err(serde::de::Error::custom(format!(
                        "unknown classic key algorithm: {}",
                        decoded[0]
                    )));
                };

                #[cfg(not(feature = "server"))]
                let algorithm = algorithm.unwrap();

                Ok(match algorithm {
                    ClassicKeyType::Curve25519 => {
                        ClassicKey::Curve25519(data.try_into().map_err(|_| {
                            serde::de::Error::custom("invalid length for Curve25519")
                        })?)
                    }
                    #[cfg(not(feature = "server"))]
                    ClassicKeyType::Unsupported(code) => ClassicKey::Unsupported { code, data },
                })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl Serialize for ClassicKey {
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
pub enum ClassicKeyType {
    Curve25519,
    #[cfg(not(feature = "server"))]
    /// Unsupported algorithm. Maybe this algorithm is not implemented
    /// during the time of this crate.
    Unsupported(u8),
}

impl ClassicKeyType {
    #[must_use]
    pub fn from_value(value: u8) -> Option<Self> {
        match value {
            0x1 => Some(Self::Curve25519),
            #[cfg(feature = "server")]
            _ => None,
            #[cfg(not(feature = "server"))]
            _ => Some(Self::Unsupported(value)),
        }
    }

    #[must_use]
    pub fn value(&self) -> u8 {
        match self {
            Self::Curve25519 => 0x1,
            #[cfg(not(feature = "server"))]
            Self::Unsupported(n) => *n,
        }
    }
}
