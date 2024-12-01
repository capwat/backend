use base64::Engine;
use capwat_api_types::e2ee::{PostQuantumKey, PostQuantumKeyType};
use capwat_error::{ext::ResultExt, Error, Result};
use ml_kem::{
    kem::{DecapsulationKey, EncapsulationKey},
    EncodedSizeUser, KemCore, MlKem768, MlKem768Params,
};
use std::fmt::Debug;
use thiserror::Error;

/// Key pairs for the Curve25519 algorithm.
#[derive(Debug, Clone)]
pub struct KeyPair {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl KeyPair {
    /// Forms a keypair from public and secret keys.
    ///
    /// It will return `None` if public and secret keys are not
    /// matched each other.
    #[must_use]
    pub fn from_keys(public_key: PublicKey, secret_key: SecretKey) -> Option<Self> {
        let actual_public_key = secret_key.0.encapsulation_key();
        if actual_public_key.as_bytes().as_slice() == public_key.0.as_bytes().as_slice() {
            return None;
        }
        Some(Self {
            public_key,
            secret_key,
        })
    }

    /// Generates a public-secret `ML-KEM768` key pair.
    #[must_use]
    pub fn generate() -> Self {
        let mut rng = crate::default_rng();
        let (decap_key, encap_key) = MlKem768::generate(&mut rng);
        Self {
            public_key: PublicKey(encap_key),
            secret_key: SecretKey(decap_key),
        }
    }

    #[must_use]
    pub fn split(self) -> (PublicKey, SecretKey) {
        (self.public_key, self.secret_key)
    }
}

#[derive(Clone)]
pub struct PublicKey(pub(super) EncapsulationKey<MlKem768Params>);

impl Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PublicKey({})", &hex::encode(&self.0.as_bytes())[0..12])
    }
}

impl PublicKey {
    /// Serializes the public key into a Capwat's [`PostQuantumKey`] type
    /// that can be serialized using [`serde`].
    ///
    /// [`PostQuantumKey`]: capwat_api_types::e2ee::PostQuantumKey
    #[must_use]
    pub fn serialize(&self) -> PostQuantumKey {
        let key_contents = self
            .0
            .as_bytes()
            .as_slice()
            .try_into()
            .expect("unexpected ml-kem768 public key goes not equal to 1184 bytes");

        PostQuantumKey::MlKem768(key_contents)
    }
}

#[derive(Clone)]
pub struct SecretKey(pub(super) DecapsulationKey<MlKem768Params>);

impl Debug for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretKey(...)")
    }
}

#[derive(Debug, Error)]
#[error("Invalid ML-KEM768 secret encoded key")]
pub struct InvalidSecretKey;

impl SecretKey {
    /// Deserializes the secret ML-KEM768 key from Base64 encoded string
    #[must_use]
    pub fn deserialize(value: &str) -> Result<Self, InvalidSecretKey> {
        let decoded = base64::engine::general_purpose::URL_SAFE
            .decode(value)
            .change_context(InvalidSecretKey)?;

        if decoded.len() != 2400 {
            return Err(Error::unknown(InvalidSecretKey))
                .attach_printable("invalid secret key length");
        }

        let inner_data = decoded
            .as_slice()
            .try_into()
            .expect("unexpected ml-kem768 secret key goes not equal to 2400 bytes");

        let inner: DecapsulationKey<MlKem768Params> = DecapsulationKey::from_bytes(inner_data);
        Ok(Self(inner))
    }

    /// Serializes the secret key into a Base64 encoded string that can
    /// be used to deserialize when needed.
    #[must_use]
    pub fn serialize(&self) -> String {
        let mut buffer = Vec::with_capacity(1 + self.0.as_bytes().len());
        buffer.push(PostQuantumKeyType::MlKem768.value());
        buffer.extend_from_slice(&self.0.as_bytes());

        base64::engine::general_purpose::URL_SAFE.encode(buffer)
    }
}
