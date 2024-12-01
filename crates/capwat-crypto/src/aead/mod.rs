mod types;
pub use self::types::*;

use capwat_error::{Error, Result};
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Could not encrypt data")]
pub struct EncryptError;

pub fn encrypt(plaintext: &[u8], key: &Key, nonce: &Nonce) -> Result<Vec<u8>, EncryptError> {
    let cipher = ChaCha20Poly1305::new(key.as_aead());
    cipher
        .encrypt(nonce.as_aead(), plaintext)
        .map_err(|_| Error::unknown(EncryptError))
}

#[derive(Debug, Error)]
#[error("Could not decrypt data")]
pub struct DecryptError;

pub fn decrypt(ciphertext: &[u8], key: &Key, nonce: &Nonce) -> Result<Vec<u8>, DecryptError> {
    let cipher = ChaCha20Poly1305::new(key.as_aead());
    cipher
        .decrypt(nonce.as_aead(), ciphertext)
        .map_err(|_| Error::unknown(DecryptError))
}
