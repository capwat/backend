use capwat_error::{ext::ResultExt, Result};
use thiserror::Error;

use crate::default_rng;

pub use rsa::pkcs1::{
    DecodeRsaPrivateKey, DecodeRsaPublicKey, EncodeRsaPrivateKey, EncodeRsaPublicKey, LineEnding,
};
pub use rsa::{RsaPrivateKey, RsaPublicKey};

#[derive(Debug, Error)]
#[error("Could not generate RSA key pair")]
pub struct GenerateKeyPairError;

pub const BITS: usize = 3072;

pub fn generate_keypair() -> Result<(RsaPublicKey, RsaPrivateKey), GenerateKeyPairError> {
    // 3072 bits should be enough for now...
    let mut rng = default_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, BITS).change_context(GenerateKeyPairError)?;
    let pub_key = RsaPublicKey::from(&priv_key);
    Ok((pub_key, priv_key))
}
