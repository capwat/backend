use crate::default_rng;
use capwat_api_types::user::UserSalt;
use rand_chacha::rand_core::RngCore;

/// Generates a random unique User salt.
#[must_use]
pub fn generate_user_salt() -> UserSalt {
    let mut buffer = [0u8; 16];
    let mut rng = default_rng();
    rng.fill_bytes(&mut buffer);

    UserSalt::from(buffer)
}

/// Generates a random unique salt.
#[must_use]
pub fn generate_salt() -> [u8; 16] {
    let mut buffer = [0u8; 16];
    let mut rng = default_rng();
    rng.fill_bytes(&mut buffer);
    buffer
}
