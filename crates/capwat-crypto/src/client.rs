use base64::{prelude::BASE64_URL_SAFE, Engine};
use capwat_api_types::{
    e2ee::{ClassicKey, PostQuantumKey},
    users::UserSalt,
};
use sha2::{Digest, Sha256};

use crate::{aead, curve25519, ml_kem768};

#[derive(Debug)]
pub struct GenerateUserMockData {
    pub salt: UserSalt,
    pub access_key_hash: String,
    pub classic_pk: ClassicKey,
    pub pqc_pk: PostQuantumKey,
    pub encrypted_classic_sk: String,
    pub encrypted_pqc_sk: String,
    pub classic_sk_nonce: String,
    pub pqc_sk_nonce: String,
}

/// It generates necessary registration data to perform a user
/// registration request to the Capwat HTTP API.
pub fn generate_mock_user_info(passphrase: &str) -> GenerateUserMockData {
    const DERIVED_KEY_BITS: usize = 512 / 8;

    // Generate our own salt and derived key
    let salt = crate::generate_salt();
    let derived_key = crate::derive_key::<DERIVED_KEY_BITS>(passphrase.as_bytes(), &salt);

    // Split derived key into halves and label them:
    // - the first portion is our access key
    // - the second portion is our AEAD key to encrypt
    //   our private classic and PQC keys.
    let (access_key, aead_key) = derived_key.split_at(derived_key.len() / 2);

    // It must be 32 bits on the AEAD key side otherwise AEAD
    // wouldn't allow us to encrypt data anyways.
    assert_eq!(aead_key.len(), 32);
    let aead_key = aead::Key::from_slice(aead_key).unwrap();

    // Generating our own access key hash, the server will determine if that
    // hash is correct based on our own calculated first portion of our derived
    // key once we're successfully registered into that service.
    let access_key_hash = {
        let mut hasher = Sha256::new();
        hasher.update(access_key);
        hex::encode(hasher.finalize())
    };

    // Generating our classic and PQC keys
    // - with Curve25519 as our classic key type
    // - and with ML-KEM768 as our PQC key type
    let (classic_pk, classic_sk) = curve25519::KeyPair::generate().split();
    let (pqc_pk, pqc_sk) = ml_kem768::KeyPair::generate().split();

    // Encrypting our classic private key with AEAD
    let classic_sk_nonce = aead::Nonce::generate();
    let encrypted_classic_sk = {
        let raw = aead::encrypt(
            classic_sk.serialize().as_bytes(),
            &aead_key,
            &classic_sk_nonce,
        )
        .unwrap();

        // we want to use Base64 with url safe variant just in case
        // if someone wants to do something using query (not recommended)
        BASE64_URL_SAFE.encode(raw)
    };
    let classic_sk_nonce = BASE64_URL_SAFE.encode(classic_sk_nonce.as_slice());

    // Encrypting our PQC private key with AEAD
    let pqc_sk_nonce = aead::Nonce::generate();
    let encrypted_pqc_sk = {
        let raw = aead::encrypt(pqc_sk.serialize().as_bytes(), &aead_key, &pqc_sk_nonce).unwrap();

        // we want to use Base64 with url safe variant just in case
        // if someone wants to do something using query (not recommended)
        BASE64_URL_SAFE.encode(raw)
    };
    let pqc_sk_nonce = BASE64_URL_SAFE.encode(pqc_sk_nonce.as_slice());

    GenerateUserMockData {
        salt: UserSalt::from(salt),
        access_key_hash,
        classic_pk: classic_pk.serialize(),
        pqc_pk: pqc_pk.serialize(),
        encrypted_classic_sk,
        encrypted_pqc_sk,
        classic_sk_nonce,
        pqc_sk_nonce,
    }
}
