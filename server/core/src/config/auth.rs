use once_cell::sync::OnceCell;
use sensitive::Sensitive;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use validator::{Validate, ValidateError};

use crate::util::MaybeGenerated;

#[derive(Deserialize, Serialize)]
pub struct Auth {
    #[serde(default = "Auth::generate_jwt_key")]
    pub(crate) jwt_key: MaybeGenerated<Sensitive<String>>,
    // Pre-computed hash
    #[serde(skip)]
    pub(crate) jwt_key_hash: OnceCell<String>,
}

impl std::fmt::Debug for Auth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Auth")
            .field("jwt_key_hash", &self.jwt_key_hash())
            .finish()
    }
}

impl Auth {
    pub fn jwt_key(&self) -> MaybeGenerated<Sensitive<&str>> {
        let value = Sensitive::new(self.jwt_key.value().as_str());
        match &self.jwt_key {
            MaybeGenerated::Generated(..) => MaybeGenerated::Generated(value),
            MaybeGenerated::Set(..) => MaybeGenerated::Set(value),
        }
    }

    /// Creates a SHA224 sum from a JWT secret.
    pub fn jwt_key_hash(&self) -> &str {
        if let Some(cache) = self.jwt_key_hash.get() {
            return cache;
        }

        let mut hasher = sha2::Sha224::new();
        hasher.update(self.jwt_key.as_bytes());

        let hash = hex::encode(hasher.finalize());
        self.jwt_key_hash.set(hash).expect("should be empty");
        self.jwt_key_hash.get().unwrap()
    }
}

impl Auth {
    pub const MIN_JWT_KEY_LENGTH: usize = 24;
    pub const MAX_JWT_KEY_LENGTH: usize = 1024;

    pub fn generate_jwt_key() -> MaybeGenerated<Sensitive<String>> {
        const CHARSET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!@#$%^&*";

        let output: String = random_string::generate(Self::MIN_JWT_KEY_LENGTH, &*CHARSET);
        MaybeGenerated::Generated(Sensitive::new(output))
    }
}

impl Default for Auth {
    fn default() -> Self {
        let auth = Self {
            jwt_key: Self::generate_jwt_key(),
            jwt_key_hash: OnceCell::new(),
        };
        let _ = auth.jwt_key_hash();
        auth
    }
}

impl Validate for Auth {
    fn validate(&self) -> Result<(), ValidateError> {
        let mut fields = ValidateError::field_builder();
        {
            let mut jwt_errs = ValidateError::msg_builder();
            if self.jwt_key.len() < Self::MIN_JWT_KEY_LENGTH {
                jwt_errs.insert("JWT secret key must have more than 16 characters");
            }
            if self.jwt_key.len() > Self::MAX_JWT_KEY_LENGTH {
                jwt_errs.insert("JWT secret key is too big");
            }
            fields.insert("jwt_key", jwt_errs.build());
        }
        fields.build().into_result()
    }
}
