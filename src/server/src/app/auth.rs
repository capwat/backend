use capwat_error::ext::{NoContextResultExt, ResultExt};
use capwat_error::{ApiErrorCategory, Error, Result};
use capwat_vfs::{OpenOptions, Vfs};
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::{Read, Write};
use std::sync::LazyLock;
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, info, warn};

use super::App;

static JWT_HEADER: LazyLock<jsonwebtoken::Header> =
    LazyLock::new(|| jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256));

impl App {
    pub fn encode_to_jwt<T: Serialize>(&self, claims: &T) -> Result<String, EncodeJwtError> {
        jsonwebtoken::encode(&JWT_HEADER, claims, &self.jwt_encode).change_context(EncodeJwtError)
    }

    pub fn decode_jwt<T: DeserializeOwned>(
        &self,
        token: &str,
        issuer: &JwtIssuer,
    ) -> Result<T, DecodeJwtError> {
        use jsonwebtoken::errors::ErrorKind;

        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.leeway = 30;
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.set_issuer(&[issuer.to_string(self)]);

        let token = token.replace(char::is_whitespace, "");
        match jsonwebtoken::decode(&token, &self.jwt_decode, &validation) {
            Ok(d) => Ok(d.claims),
            Err(error) => match error.kind() {
                ErrorKind::Json(..) | ErrorKind::InvalidIssuer | ErrorKind::InvalidToken => {
                    Err(Error::new(ApiErrorCategory::AccessDenied, DecodeJwtError))
                }
                ErrorKind::ExpiredSignature => {
                    Err(Error::new(ApiErrorCategory::ExpiredToken, DecodeJwtError))
                }
                _ => Err(Error::unknown_generic(error).change_context(DecodeJwtError)),
            },
        }
    }
}

#[derive(Debug, Error)]
#[error("Failed to decode JWT")]
pub struct DecodeJwtError;

#[derive(Debug, Error)]
#[error("Failed to encode claims as JWT")]
pub struct EncodeJwtError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JwtIssuer {
    Login,
}

impl JwtIssuer {
    #[allow(unused)]
    #[must_use]
    pub fn to_string(&self, app: &App) -> String {
        let prefix = "";
        match self {
            Self::Login => format!("{prefix}login"),
        }
    }
}

impl App {
    pub(super) fn setup_jwt_keys(
        config: &capwat_config::Server,
        vfs: &Vfs,
    ) -> Result<(EncodingKey, DecodingKey)> {
        use capwat_crypto::rsa::{
            self, DecodeRsaPrivateKey, EncodeRsaPrivateKey, EncodeRsaPublicKey, RsaPrivateKey,
        };

        fn read_jwt_priv_key_file(
            config: &capwat_config::server::Jwt,
            vfs: &Vfs,
        ) -> Result<RsaPrivateKey> {
            let mut buffer = String::with_capacity(rsa::BITS);
            let now = Instant::now();

            debug!("reading JWT private key file...");

            if !vfs.is_using_std_backend() {
                // generate automatically then.
                let new_priv = rsa::generate_keypair()?.1;
                let elapsed = now.elapsed();
                debug!(?elapsed, "reading JWT private key file done");
                return Ok(new_priv);
            }

            let mut file = OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(vfs, &config.private_key_file)?;

            let bytes_read = file.read_to_string(&mut buffer)?;
            let rsa_key = if bytes_read > 0 {
                RsaPrivateKey::from_pkcs1_pem(&buffer)?
            } else {
                warn!("JWT private key file is missing. Generating new JWT private key (this will invalidate all user sessions)...");
                let new_priv_key = rsa::generate_keypair()?.1;
                file.write_all(new_priv_key.to_pkcs1_pem(rsa::LineEnding::LF)?.as_bytes())?;

                info!(
                    "created JWT private key file: {}",
                    config.private_key_file.display()
                );
                new_priv_key
            };

            let elapsed = now.elapsed();
            debug!(?elapsed, "reading JWT private key file done");

            Ok(rsa_key)
        }

        let jwt = &config.auth.jwt;
        let priv_key = read_jwt_priv_key_file(jwt, vfs)
            .attach_printable("could not generate JWT key files")?;

        let priv_key_buffer = priv_key.to_pkcs1_pem(rsa::LineEnding::LF)?;
        let pub_key_buffer = priv_key.to_public_key().to_pkcs1_pem(rsa::LineEnding::LF)?;

        let enc = EncodingKey::from_rsa_pem(priv_key_buffer.as_bytes())
            .attach_printable("could not read RSA PEM of private key file")?;

        let dec = DecodingKey::from_rsa_pem(pub_key_buffer.as_bytes())
            .attach_printable("unexpected to unable to read RSA PEM from public key")?;

        Ok((enc, dec))
    }
}
