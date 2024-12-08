use sha2::{Digest, Sha256};

use crate::curve25519;

/// Derives a unique key from a classic and post-quantum secret keys
/// by hashing both keys using SHA-256 algorithm.
#[must_use]
pub fn derive_key_from_keypairs(classic: &curve25519::SecretKey) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(classic.as_bytes());
    hasher.finalize().try_into().unwrap()
}

/// Derives a unique key with a number of elements from a
/// passphrase and salt array.
#[must_use]
pub fn derive_from_passphrase<const N: usize>(passphrase: &[u8], salt: &[u8]) -> [u8; N] {
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
