mod key;
pub use self::key::{KeyPair, PublicKey, SecretKey};

use crate::aead;
use capwat_error::{ext::ResultExt, Error, Result};
use ml_kem::kem::{Decapsulate, Encapsulate};
use sha2::{Digest, Sha256};

/// Attempts to encrypt plaintext into a ciphertext from the recipient's public key
/// using `Chacha20Poly1305` as the symmetrical encryption scheme.
///
/// It returns both the data ciphertext and the ML-KEM ciphertext to decrypt the
/// ciphertext if needed to the recipient.
///
/// It will throw an error if:
/// - The resulting key may cause conflict to the system and potentially
///   pose a security risk for both users (the sender and the recipient).
pub fn encrypt(
    recipient: &PublicKey,
    plaintext: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), aead::EncryptError> {
    let mut rng = crate::default_rng();
    let (ml_ciphertext, shared_secret) = recipient
        .0
        .encapsulate(&mut rng)
        .map_err(|_| Error::unknown(aead::EncryptError))?;

    let derived_key = derive_aead_key(&shared_secret);
    let ciphertext =
        aead::encrypt(plaintext, &derived_key).map_err(|_| Error::unknown(aead::EncryptError))?;

    Ok((ciphertext, ml_ciphertext.to_vec()))
}

pub fn decrypt(
    recipient: &SecretKey,
    ml_ciphertext: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, aead::DecryptError> {
    let ml_ciphertext = ml_ciphertext
        .try_into()
        .map_err(|_| Error::unknown(aead::DecryptError))
        .attach_printable("invalid ML-KEM ciphertext")?;

    let shared_secret = recipient
        .0
        .decapsulate(ml_ciphertext)
        .map_err(|_| Error::unknown(aead::DecryptError))?;

    let derived_key = derive_aead_key(&shared_secret);
    aead::decrypt(&ciphertext[12..], &derived_key).map_err(|_| Error::unknown(aead::DecryptError))
}

/// Derives an AEAD key from shared secret key.
fn derive_aead_key(shared_key: &[u8]) -> aead::Key {
    let mut hasher = Sha256::new();
    hasher.update(shared_key);

    let result = hasher.finalize();
    aead::Key::from_slice(&result).unwrap()
}
