use crate::default_rng;
use rand_chacha::rand_core::RngCore;

pub type CapwatSaltArray = [u8; 16];

/// Generates a random unique [Capwat user salt].
///
/// [Capwat user salt]: CapwatSaltArray
#[must_use]
pub fn generate_salt() -> CapwatSaltArray {
    let mut buffer = [0u8; 16];
    let mut rng = default_rng();
    rng.fill_bytes(&mut buffer);
    buffer
}
