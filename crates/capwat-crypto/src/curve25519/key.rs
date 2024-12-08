use capwat_api_types::encrypt::{ClassicKey, ClassicKeyType};
use std::fmt::Debug;

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
        let actual_public_key = x25519_dalek::PublicKey::from(&secret_key.0);
        if actual_public_key.as_bytes() == public_key.as_bytes() {
            return None;
        }
        Some(Self {
            public_key,
            secret_key,
        })
    }

    /// Generates a public-secret Curve25519 key pair.
    #[must_use]
    pub fn generate() -> Self {
        let mut rng = crate::default_rng();
        let secret_key = x25519_dalek::StaticSecret::random_from_rng(&mut rng);
        let public_key = x25519_dalek::PublicKey::from(&secret_key);

        Self {
            public_key: PublicKey(public_key),
            secret_key: SecretKey(secret_key),
        }
    }

    #[must_use]
    pub fn split(self) -> (PublicKey, SecretKey) {
        (self.public_key, self.secret_key)
    }
}

#[derive(Clone)]
pub struct PublicKey(pub(super) x25519_dalek::PublicKey);

impl Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PublicKey({})", &hex::encode(self.as_bytes())[0..12])
    }
}

impl PublicKey {
    /// Creates a new public key from an [API key] (`ClassicKey` in [`capwat_api_types`]).
    ///
    /// It will return `None` if the key type is not `Curve25519`.
    ///
    /// [API key]: capwat_api_types::e2ee::ClassicKey
    #[must_use]
    pub fn from_api(key: ClassicKey) -> Option<Self> {
        match key.as_key_type() {
            ClassicKeyType::Curve25519 => {
                assert_eq!(key.as_bytes().len(), ClassicKeyType::CURVE25519_SIZE);

                let sized: [u8; 32] = key.as_bytes().try_into().unwrap();
                let public_key = x25519_dalek::PublicKey::from(sized);

                Some(Self(public_key))
            }
            #[cfg(not(feature = "server"))]
            ClassicKeyType::Unsupported(..) => None,
        }
    }

    /// Gets the raw bytes of a public Curve25519 key.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    /// Serializes the public key into a Capwat's [`ClassicKey`] type
    /// that can be serialized using [`serde`].
    ///
    /// [`ClassicKey`]: capwat_api_types::e2ee::ClassicKey
    #[must_use]
    pub fn serialize(&self) -> ClassicKey {
        ClassicKey::new(ClassicKeyType::Curve25519, self.as_bytes()).unwrap()
    }
}

#[derive(Clone)]
pub struct SecretKey(pub(super) x25519_dalek::StaticSecret);

impl Debug for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretKey(...)")
    }
}

impl SecretKey {
    /// Creates a new secret key from an [API key] (`ClassicKey` in [`capwat_api_types`]).
    ///
    /// It will return `None` if the key type is not `Curve25519`.
    ///
    /// [API key]: capwat_api_types::e2ee::ClassicKey
    #[must_use]
    pub fn from_api(key: ClassicKey) -> Option<Self> {
        match key.as_key_type() {
            ClassicKeyType::Curve25519 => {
                assert_eq!(key.as_bytes().len(), ClassicKeyType::CURVE25519_SIZE);

                let sized: [u8; 32] = key.as_bytes().try_into().unwrap();
                let public_key = x25519_dalek::StaticSecret::from(sized);

                Some(Self(public_key))
            }
            #[cfg(not(feature = "server"))]
            ClassicKeyType::Unsupported(..) => None,
        }
    }

    /// Gets the raw bytes of a public Curve25519 key.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    /// Serializes the secret key into a Base64 encoded string that can
    /// be used to deserialize when needed.
    #[must_use]
    pub fn serialize(&self) -> String {
        ClassicKey::new(ClassicKeyType::Curve25519, self.as_bytes())
            .unwrap()
            .to_string()
    }
}
