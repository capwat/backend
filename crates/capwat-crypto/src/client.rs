use capwat_api_types::encrypt::ClassicKey;
use capwat_api_types::user::UserSalt;
use capwat_api_types::util::EncodedBase64;

use crate::derive::derive_from_passphrase;
use crate::{aead, curve25519};

#[derive(Debug)]
pub struct RegisterUserParams {
    pub salt: UserSalt,
    pub access_key_hash: EncodedBase64,
    pub encrypted_symmetric_key: EncodedBase64,

    pub public_key: ClassicKey,
    pub encrypted_secret_key: EncodedBase64,
}

/// It generates necessary registration data to perform a user
/// registration request to the Capwat HTTP API.
pub fn generate_register_user_params(passphrase: &str) -> RegisterUserParams {
    // this technique is inspired from Bitwarden because I cannot make
    // my own password derive stuff from scratch.
    const DERIVED_KEY_BYTES: usize = 512 / 8;
    const SHA256_BYTES: usize = 512 / 8;

    // Generate our own salt and own derived key (512 bits)
    let salt = crate::salt::generate_user_salt();
    let derived_key =
        derive_from_passphrase::<DERIVED_KEY_BYTES>(passphrase.as_bytes(), &salt.as_slice());

    // Then, we're going to hash the derived key another so that
    // we can send through the database.
    let access_key_hash = EncodedBase64::from_bytes(derive_from_passphrase::<SHA256_BYTES>(
        &derived_key,
        passphrase.as_bytes(),
    ));

    // Derive another one to create AEAD key to encrypt our symmetric key
    let symmetric_key_aead_key = aead::Key::from_slice(&crate::hash::sha256(
        &derive_from_passphrase::<SHA256_BYTES>(&derived_key, salt.as_slice()),
    ))
    .unwrap();

    let symmetric_key = aead::Key::new();

    // Generating our classic keys
    let (classic_pk, classic_sk) = curve25519::KeyPair::generate().split();
    let encrypted_secret_key = {
        let raw = aead::encrypt(classic_sk.serialize().as_bytes(), &symmetric_key).unwrap();
        EncodedBase64::from_bytes(raw)
    };

    let encrypted_symmetric_key = {
        let raw = aead::encrypt(symmetric_key.as_slice(), &symmetric_key_aead_key).unwrap();
        EncodedBase64::from_bytes(raw)
    };

    RegisterUserParams {
        salt,
        access_key_hash,
        encrypted_symmetric_key,
        public_key: classic_pk.serialize(),
        encrypted_secret_key,
    }
}
