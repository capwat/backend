use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use capwat_error::{ext::ResultExt, Result};
use std::sync::LazyLock;
use thiserror::Error;

static CONTEXT: LazyLock<Argon2<'static>> = LazyLock::new(|| {
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::DEFAULT,
    )
});

#[derive(Debug, Error)]
#[error("Failed to generate password hash")]
pub struct HashPasswordError;

pub fn hash(password: impl AsRef<[u8]>) -> Result<String, HashPasswordError> {
    let salt = SaltString::generate(&mut crate::default_rng());
    let password_hash = CONTEXT
        .hash_password(password.as_ref(), &salt)
        .change_context(HashPasswordError)?;

    Ok(password_hash.to_string())
}

#[derive(Debug, Error)]
#[error("Failed to verify password")]
pub struct VerifyPasswordError;

pub fn verify(password: &[u8], hash: &str) -> Result<bool, VerifyPasswordError> {
    let hash = PasswordHash::new(hash)
        .change_context(VerifyPasswordError)
        .attach_printable("could not parse password hash")?;

    match CONTEXT.verify_password(password, &hash) {
        Ok(..) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(error) => Err(error).change_context(VerifyPasswordError),
    }
}
