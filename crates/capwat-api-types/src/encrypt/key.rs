use base64::{prelude::BASE64_URL_SAFE, Engine};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;
use std::str::FromStr;
use thiserror::Error;

use crate::internal::Sealed;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Key<T: KeyType> {
    inner: T,
    data: Vec<u8>,
}

impl<T: KeyType> Key<T> {
    #[must_use]
    pub fn new(key_type: T, data: impl AsRef<[u8]>) -> Option<Self> {
        let data = data.as_ref();
        key_type.validate_len(data).map(|_| Self {
            inner: key_type,
            data: data.to_vec(),
        })
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    #[must_use]
    pub fn as_key_type(&self) -> &T {
        &self.inner
    }
}

impl<T: KeyType> Debug for Key<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}({})",
            self.inner,
            BASE64_URL_SAFE.encode(&self.data)
        )
    }
}

impl<T: KeyType> Display for Key<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = Vec::with_capacity(1 + self.data.len());
        buffer.push(self.inner.encode());
        buffer.extend_from_slice(&self.data);

        Display::fmt(&BASE64_URL_SAFE.encode(&buffer), f)
    }
}

#[derive(Debug, Error)]
#[error("could not parse {class}: {kind}")]
pub struct KeyParseError {
    class: &'static str,
    kind: KeyParseErrorKind,
}

#[derive(Debug)]
enum KeyParseErrorKind {
    Encoding(base64::DecodeError),
    InvalidLength(String),
    InvalidValue(u8),
    MissingAlgorithmHeader,
}

impl Display for KeyParseErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Encoding(n) => Display::fmt(n, f),
            Self::InvalidLength(c) => write!(f, "invalid length for {c:?}"),
            Self::InvalidValue(n) => write!(f, "no code for {n}"),
            Self::MissingAlgorithmHeader => f.write_str("missing algorithm header"),
        }
    }
}

impl<T: KeyType> FromStr for Key<T> {
    type Err = KeyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw_bytes = BASE64_URL_SAFE.decode(s).map_err(|e| KeyParseError {
            kind: KeyParseErrorKind::Encoding(e),
            class: T::serde_name(),
        })?;

        let code = *raw_bytes.get(0).ok_or_else(|| KeyParseError {
            class: T::serde_name(),
            kind: KeyParseErrorKind::MissingAlgorithmHeader,
        })?;

        let inner = T::decode(code).ok_or_else(|| KeyParseError {
            class: T::serde_name(),
            kind: KeyParseErrorKind::InvalidValue(code),
        })?;

        let data = &raw_bytes[1..];
        inner.validate_len(data).ok_or_else(|| KeyParseError {
            class: T::serde_name(),
            kind: KeyParseErrorKind::InvalidLength(format!("{inner:?}")),
        })?;

        Ok(Key {
            inner,
            data: raw_bytes[1..].to_vec(),
        })
    }
}

impl<'de, T: KeyType> serde::Deserialize<'de> for Key<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<T: KeyType>(PhantomData<T>);

        impl<'de, T: KeyType> serde::de::Visitor<'de> for Visitor<T> {
            type Value = Key<T>;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(T::serde_name())
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Key::<T>::from_str(v).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(Visitor(PhantomData::<T>))
    }
}

impl<T: KeyType> serde::Serialize for Key<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

pub trait KeyType: Debug + PartialEq + Eq + Hash + Sealed + Sized {
    #[doc(hidden)]
    fn serde_name() -> &'static str;
    #[doc(hidden)]
    fn decode(code: u8) -> Option<Self>;
    #[doc(hidden)]
    fn encode(&self) -> u8;

    #[allow(unused)]
    #[doc(hidden)]
    fn validate_len(&self, data: &[u8]) -> Option<()> {
        None
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ClassicKeyType {
    Curve25519,
    #[cfg(not(feature = "server"))]
    Unknown(u8),
}

impl ClassicKeyType {
    pub const CURVE25519_ID: u8 = 1;
    pub const CURVE25519_SIZE: usize = 32;
}

pub type ClassicKey = Key<ClassicKeyType>;

impl Debug for ClassicKeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Curve25519 => write!(f, "Curve25519"),
            #[cfg(not(feature = "server"))]
            Self::Unknown(code) => write!(f, "Unknown({code})"),
        }
    }
}

impl KeyType for ClassicKeyType {
    fn serde_name() -> &'static str {
        "Capwat user classic key"
    }

    fn decode(code: u8) -> Option<ClassicKeyType> {
        match code {
            0x1 => Some(Self::Curve25519),
            _ => None,
        }
    }

    fn encode(&self) -> u8 {
        match self {
            Self::Curve25519 => Self::CURVE25519_ID,
            #[cfg(not(feature = "server"))]
            Self::Unknown(n) => *n,
        }
    }

    fn validate_len(&self, data: &[u8]) -> Option<()> {
        match self {
            Self::Curve25519 => {
                if data.len() == Self::CURVE25519_SIZE {
                    Some(())
                } else {
                    None
                }
            }
            #[cfg(not(feature = "server"))]
            Self::Unknown(..) => None,
        }
    }
}

impl Sealed for ClassicKeyType {}

#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg(feature = "experimental")]
pub enum PostQuantumKeyType {
    MlKem768Public,
    MlKem768Private,
    #[cfg(not(feature = "server"))]
    Unknown(u8),
}

#[cfg(feature = "experimental")]
impl PostQuantumKeyType {
    pub const MLKEM_768_PUBLIC_ID: u8 = 1;
    pub const MLKEM_768_PUBLIC_SIZE: usize = 1184;

    pub const MLKEM_768_PRIVATE_ID: u8 = 2;
    pub const MLKEM_768_PRIVATE_SIZE: usize = 2400;
}

#[cfg(feature = "experimental")]
pub type PostQuantumKey = Key<PostQuantumKeyType>;

#[cfg(feature = "experimental")]
impl Debug for PostQuantumKeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MlKem768Public => write!(f, "ML-KEM768Public"),
            Self::MlKem768Private => write!(f, "ML-KEM768Private"),
            #[cfg(not(feature = "server"))]
            Self::Unknown(code) => write!(f, "Unknown({code})"),
        }
    }
}

#[cfg(feature = "experimental")]
impl KeyType for PostQuantumKeyType {
    fn serde_name() -> &'static str {
        "Capwat user post-quantum key"
    }

    fn decode(code: u8) -> Option<PostQuantumKeyType> {
        match code {
            Self::MLKEM_768_PUBLIC_ID => Some(Self::MlKem768Public),
            Self::MLKEM_768_PRIVATE_ID => Some(Self::MlKem768Private),
            _ => None,
        }
    }

    fn encode(&self) -> u8 {
        match self {
            Self::MlKem768Public => Self::MLKEM_768_PUBLIC_ID,
            Self::MlKem768Private => Self::MLKEM_768_PRIVATE_ID,
            #[cfg(not(feature = "server"))]
            Self::Unknown(n) => *n,
        }
    }

    fn validate_len(&self, data: &[u8]) -> Option<()> {
        match self {
            Self::MlKem768Public => {
                if data.len() == Self::MLKEM_768_PUBLIC_SIZE {
                    Some(())
                } else {
                    None
                }
            }
            Self::MlKem768Private => {
                if data.len() == Self::MLKEM_768_PRIVATE_SIZE {
                    Some(())
                } else {
                    None
                }
            }
            #[cfg(not(feature = "server"))]
            Self::Unknown(..) => None,
        }
    }
}

#[cfg(feature = "experimental")]
impl Sealed for PostQuantumKeyType {}
