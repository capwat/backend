mod key;
pub use self::key::{KeyPair, PublicKey, SecretKey};

use crate::aead;
use capwat_error::{ext::ResultExt, Error, Result};
use sha2::{Digest, Sha256};

/// Attempts to encrypt plaintext into a ciphertext from the recipient's public key
/// using `Chacha20Poly1305` as the symmetrical encryption scheme.
///
/// It returns both the ciphertext and the sender's ephemeral public key
/// to decrypt the ciphertext if needed to the recipient.
///
/// It will throw an error if:
/// - The resulting key may cause conflict to the system and potentially
///   pose a security risk for both users (the sender and the recipient).
pub fn encrypt(
    recipient: &PublicKey,
    plaintext: &[u8],
) -> Result<(Vec<u8>, PublicKey), aead::EncryptError> {
    let (ephemeral_public_key, ephemeral_secret_key) = KeyPair::generate().split();
    let shared_secret = ephemeral_secret_key.0.diffie_hellman(&recipient.0);

    // It may be a security risk if we failed to be careful when
    // handling Diffie-Hellman keys. :)
    if !shared_secret.was_contributory() {
        return Err(Error::unknown(aead::EncryptError))
            .attach_printable("conflicting public recipient key");
    }

    let derived_key = derive_aead_key(&shared_secret);
    let ciphertext =
        aead::encrypt(plaintext, &derived_key).map_err(|_| Error::unknown(aead::EncryptError))?;

    Ok((ciphertext, ephemeral_public_key))
}

pub fn decrypt(
    recipient: &SecretKey,
    ephemeral_sender_key: &PublicKey,
    ciphertext: &[u8],
) -> Result<Vec<u8>, aead::DecryptError> {
    let shared_secret = recipient.0.diffie_hellman(&ephemeral_sender_key.0);

    // It may be a security risk if we failed to be careful when
    // handling Diffie-Hellman keys. :)
    if !shared_secret.was_contributory() {
        return Err(Error::unknown(aead::DecryptError)
            .attach_printable("conflicting recipient and sender's ephemeral keys"));
    }

    let derived_key = derive_aead_key(&shared_secret);
    aead::decrypt(&ciphertext[12..], &derived_key).map_err(|_| Error::unknown(aead::DecryptError))
}

/// Derives an AEAD key from shared secret key.
fn derive_aead_key(shared_key: &x25519_dalek::SharedSecret) -> aead::Key {
    let mut hasher = Sha256::new();
    hasher.update(shared_key.as_bytes());

    let result = hasher.finalize();
    aead::Key::from_slice(&result).unwrap()
}
