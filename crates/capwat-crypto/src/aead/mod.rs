mod types;
pub use self::types::*;

use capwat_error::{Error, Result};
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Could not encrypt data")]
pub struct EncryptError;

pub fn encrypt(plaintext: &[u8], key: &Key) -> Result<Vec<u8>, EncryptError> {
    let cipher = ChaCha20Poly1305::new(key.as_aead());
    let nonce = Nonce::generate();

    let mut ciphertext = nonce.as_slice().to_vec();
    ciphertext.extend_from_slice(
        &cipher
            .encrypt(nonce.as_aead(), plaintext)
            .map_err(|_| Error::unknown(EncryptError))?,
    );

    Ok(ciphertext)
}

#[derive(Debug, Error)]
#[error("Could not decrypt data")]
pub struct DecryptError;

pub fn decrypt(ciphertext: &[u8], key: &Key) -> Result<Vec<u8>, DecryptError> {
    // ciphertext is too small to decrypt it since we need to get
    // the nonce directly from the ciphertext.
    if ciphertext.len() < 12 {
        return Err(Error::unknown(DecryptError));
    }

    let nonce = Nonce::from_slice(&ciphertext[0..12]).unwrap();
    let cipher = ChaCha20Poly1305::new(key.as_aead());
    cipher
        .decrypt(nonce.as_aead(), &ciphertext[12..])
        .map_err(|_| Error::unknown(DecryptError))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_encrypt_and_decrypt() {
        let plaintext = b"Hello, World!";
        let key = Key::new();

        let ciphertext = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }
}
