use once_cell::sync::OnceCell;
use serde::Deserialize;
use sha2::Digest;
use validator::{Validate, ValidateError};

use crate::util::{MaybeGenerated, Sensitive};

#[derive(Deserialize)]
pub struct Auth {
  pub(crate) jwt_key: MaybeGenerated<Sensitive<String>>,
  // Pre-computed hash
  #[serde(skip)]
  pub(crate) jwt_key_hash: OnceCell<String>,
}

impl Auth {
  #[must_use]
  pub fn jwt_key(&self) -> MaybeGenerated<Sensitive<&str>> {
    let value = Sensitive::new(self.jwt_key.value().as_str());
    match &self.jwt_key {
      MaybeGenerated::Generated(..) => MaybeGenerated::Generated(value),
      MaybeGenerated::Set(..) => MaybeGenerated::Set(value),
    }
  }

  #[must_use]
  pub fn jwt_key_hash(&self) -> &str {
    if let Some(cache) = self.jwt_key_hash.get() {
      return cache;
    }

    let mut hasher = sha2::Sha224::new();
    hasher.update(self.jwt_key.as_bytes());

    let hash = hex::encode(hasher.finalize());
    self.jwt_key_hash.set(hash).expect("should be empty");

    // It is already set above
    #[allow(clippy::unwrap_used)]
    self.jwt_key_hash.get().unwrap()
  }
}

impl Auth {
  pub const MIN_JWT_KEY_LENGTH: usize = 24;
  pub const MAX_JWT_KEY_LENGTH: usize = 1024;

  #[must_use]
  pub fn generate_jwt_key() -> MaybeGenerated<Sensitive<String>> {
    const CHARSET: &str =
      "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!@#$%^&*";

    // Recommended JWT secret key length, it's long but it's worth it.
    let output: String = random_string::generate(64, CHARSET);
    MaybeGenerated::Generated(Sensitive::new(output))
  }
}

impl Default for Auth {
  /// Creates a default [`Auth`] struct.
  ///
  /// When this function is called, it will generate a new
  /// randomized JWT secret key from [`Self::generate_jwt_key`].
  fn default() -> Self {
    let auth =
      Self { jwt_key: Self::generate_jwt_key(), jwt_key_hash: OnceCell::new() };

    // We need to precompute the hash upon generating a JWT key
    #[allow(clippy::let_underscore_must_use)]
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

      let has_valid_chars = self.jwt_key.chars().any(|v| {
        v.is_alphabetic()
          || matches!(v, '!' | '@' | '#' | '$' | '%' | '^' | '&' | '*')
      });

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
  use sha2::{Digest, Sha224};
  use validator::Validate;

  #[test]
  fn test_jwt_key_hash() {
    let auth = Auth::default();

    let mut hasher = Sha224::new();
    hasher.update(auth.jwt_key.as_bytes());

    let hash = hex::encode(hasher.finalize());
    assert_eq!(hash, auth.jwt_key_hash());
  }

  #[test]
  fn test_generated_jwt_key() {
    let key = Auth::generate_jwt_key();
    assert!(key.len() == 64, "Generated key is not equal to 64 characters");

    let auth = Auth::default();
    assert_eq!(auth.validate(), Ok(()));
  }
}
