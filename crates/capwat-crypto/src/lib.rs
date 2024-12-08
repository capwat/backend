// Capwat utilizes the hybrid of pre-quantum cryptography and post-quantum cryptography
// to utilize the well-tested encryption algorithms such as `X25519` and `RSA` but
// also provide extra security measure against quantum computers with encryption
// algorithms like `ML-KEM`.
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;

pub mod hash;
pub mod salt;

// Re-exports of capwat-error because why not.
pub use capwat_error::{Error, Result};

/// This module is for AEAD encryption.
pub mod aead;
#[cfg(feature = "server")]
pub mod argon2;
pub mod curve25519;
pub mod derive;
pub mod rsa;

// Post-quantum encryption is not our priority at the moment...
// pub mod ml_kem768;

/// Simulates the client protocol for the Capwat API such as
/// key generation, encryption and so on.
#[cfg(feature = "server")]
pub mod client;

/// Useful cryptography-related utilities for [`Future`]s.
///
/// [`Future`]: std::future::Future
#[cfg(feature = "server")]
pub mod future;

pub mod base64 {
    use base64::{prelude::BASE64_URL_SAFE, Engine};

    #[must_use]
    pub fn encode(data: impl AsRef<[u8]>) -> String {
        BASE64_URL_SAFE.encode(data)
    }
}
pub use ::hex;

/// Gets the default RNG (random number generator) for `capwat-server`
/// which is [`ChaCha20Rng`].
pub fn default_rng() -> ChaCha20Rng {
    // Our default RNG (random number generator) which is [`ChaCha20Rng`]. This
    // random number generator allows to generate secure numbers but also to
    // provides good performance which is critical for server use case.
    ChaCha20Rng::from_entropy()
}
