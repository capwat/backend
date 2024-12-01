use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use rand_chacha::rand_core::RngCore;
use sha2::{Digest, Sha256};
use std::fmt::{Debug, Display};
use std::str::FromStr;
use thiserror::Error;

use crate::default_rng;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key([u8; 32]);

impl Key {
    #[must_use]
    pub fn new() -> Self {
        let mut bytes = [0u8; 32];
        let mut rng = default_rng();
        rng.fill_bytes(&mut bytes);

        Self(bytes)
    }

    #[must_use]
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        let slice: [u8; 32] = slice.try_into().ok()?;
        Some(Self(slice))
    }

    pub(super) fn as_aead(&self) -> &chacha20poly1305::Key {
        chacha20poly1305::Key::from_slice(&self.0)
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut hasher = Sha256::new();
        hasher.update(&self.0);

        let hash = hex::encode(hasher.finalize());
        write!(f, "Key({})", &hash[0..10])
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nonce([u8; 12]);

impl Nonce {
    #[must_use]
    pub fn generate() -> Self {
        let mut bytes = [0u8; 12];
        let mut rng = default_rng();
        rng.fill_bytes(&mut bytes);

        Self(bytes)
    }

    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        let slice: [u8; 12] = slice.try_into().ok()?;
        Some(Self(slice))
    }

    pub(super) fn as_aead(&self) -> &chacha20poly1305::Nonce {
        chacha20poly1305::Nonce::from_slice(&self.0)
    }
}

impl Debug for Nonce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Nonce({})", hex::encode(&self.0))
    }
}

impl Display for Nonce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&URL_SAFE.encode(self.0), f)
    }
}

#[derive(Debug, Error)]
pub enum DecodeNonceError {
    #[error("failed to decode nonce: {0}")]
    Base64(base64::DecodeError),
    #[error("invalid length for a nonce")]
    InvalidLength,
}

impl FromStr for Nonce {
    type Err = DecodeNonceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = URL_SAFE.decode(s).map_err(DecodeNonceError::Base64)?;
        if decoded.len() == 12 {
            let mut buffer = [0u8; 12];
            buffer.copy_from_slice(&decoded);

            Ok(Self(buffer))
        } else {
            Err(DecodeNonceError::InvalidLength)
        }
    }
}
