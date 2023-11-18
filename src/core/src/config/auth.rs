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
  /// Gets the raw value of JWT secret key
  pub fn jwt_key(&self) -> MaybeGenerated<Sensitive<&str>> {
    let value = Sensitive::new(self.jwt_key.value().as_str());
    match &self.jwt_key {
      MaybeGenerated::Generated(..) => MaybeGenerated::Generated(value),
      MaybeGenerated::Set(..) => MaybeGenerated::Set(value),
    }
  }

  /// Generates or get the SHA224 hash of a JWT secret key
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

  /// Generates a new JWT secret key with alphabetic and special
  /// characters are randomized and scrambled into 24 characters.
  /// (minimum amount of characters required for a JWT secret key for Whim)
  pub fn generate_jwt_key() -> MaybeGenerated<Sensitive<String>> {
    const CHARSET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!@#$%^&*";

    let output: String = random_string::generate(Self::MIN_JWT_KEY_LENGTH, &*CHARSET);
    MaybeGenerated::Generated(Sensitive::new(output))
  }
}

impl Default for Auth {
  /// Creates a default [`Auth`] struct.
  ///
  /// When this function is called, it will generate a new
  /// randomized JWT secret key from [`Self::generate_jwt_key`].
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
  /// When this [struct](Auth) performs a validation, it checks
  /// for if the `jwt_key` is within 24 up to 1024 characters.
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

      let has_valid_chars = self
        .jwt_key
        .chars()
        .any(|v| v.is_alphabetic() || matches!(v, '!' | '@' | '#' | '$' | '%' | '^' | '&' | '*'));

      if !has_valid_chars {
        jwt_errs
          .insert("JWT secret key must contain alphabetic and ASCII-compatible symbol characters");
      }
      fields.insert("jwt_key", jwt_errs.build());
    }
    fields.build().into_result()
  }
}

#[cfg(test)]
mod tests {
  use super::Auth;
  use validator::Validate;

  #[test]
  fn test_generated_jwt_key() {
    let key = Auth::generate_jwt_key();
    assert!(
      key.len() == Auth::MIN_JWT_KEY_LENGTH,
      "Generated key is not equal to the minimum JWT key length"
    );

    let auth = Auth::default();
    assert_eq!(auth.validate(), Ok(()));
  }
}
