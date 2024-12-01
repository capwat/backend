// Capwat utilizes the hybrid of pre-quantum cryptography and post-quantum cryptography
// to utilize the well-tested encryption algorithms such as `X25519` and `RSA` but
// also provide extra security measure against quantum computers with encryption
// algorithms like `ML-KEM`.
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::time::SystemTime;

mod salt;
pub use self::salt::{generate_salt, CapwatSaltArray};

// Re-exports of capwat-error because why not.
pub use capwat_error::{Error, Result};

/// This module is for AEAD encryption.
pub mod aead;
#[cfg(feature = "server")]
pub mod argon2;
pub mod curve25519;
pub mod ml_kem768;

/// Simulates the client protocol for the Capwat API such as
/// key generation, encryption and so on.
#[cfg(feature = "server")]
pub mod client;

/// Useful cryptography-related utilities for [`Future`]s.
///
/// [`Future`]: std::future::Future
#[cfg(feature = "server")]
pub mod future;

/// Derives a unique key with a number of elements from a
/// passphrase and salt array.
#[must_use]
pub fn derive_key<const N: usize>(passphrase: &[u8], salt: &CapwatSaltArray) -> [u8; N] {
    let mut buffer = [0u8; N];
    scrypt::scrypt(
        passphrase,
        salt,
        &scrypt::Params::recommended(),
        &mut buffer,
    )
    .unwrap();
    buffer
}

/// Gets the default RNG (random number generator) for `capwat-server`
/// which is [`ChaCha20Rng`].
pub fn default_rng() -> ChaCha20Rng {
    // Our default RNG (random number generator) which is [`ChaCha20Rng`]. This
    // random number generator allows to generate secure numbers but also to
    // provides good performance which is critical for server use case.
    let seed = SystemTime::UNIX_EPOCH.elapsed().unwrap_or_default();
    ChaCha20Rng::seed_from_u64(seed.as_millis() as u64)
}
