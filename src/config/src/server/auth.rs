use capwat_error::{
    ext::{NoContextResultExt, ResultExt},
    Result,
};
use capwat_macros::ConfigParts;
use capwat_utils::env;
use doku::Document;
use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

use crate::vars;

#[derive(Debug, Document, ConfigParts)]
#[config(attr(derive(Debug, Default, Deserialize)))]
#[config(attr(serde(default, rename_all = "kebab-case")))]
pub struct Auth {
    /// Configuration for authenticating users with JSON Web Tokens (JWTs).
    ///
    /// Please refer to the documentation for JWTs to see its documentation.
    #[config(as_struct, as_type = "PartialJwt")]
    pub jwt: Jwt,
}

#[derive(Debug, Error)]
#[error("Could not load configuration for authentication")]
pub struct AuthLoadError;

impl Auth {
    pub(crate) fn from_partial(partial: PartialAuth) -> Result<Self, AuthLoadError> {
        Ok(Self {
            jwt: Jwt::from_partial(partial.jwt).change_context(AuthLoadError)?,
        })
    }
}

impl PartialAuth {
    pub(crate) fn from_env() -> Result<Self, AuthLoadError> {
        let jwt = PartialJwt::from_env().change_context(AuthLoadError)?;
        Ok(Self { jwt })
    }
}

#[derive(Debug, Document, ConfigParts)]
#[config(attr(derive(Debug, Deserialize)))]
#[config(attr(serde(default, rename_all = "kebab-case")))]
pub struct Jwt {
    /// **Environment variable**: `CAPWAT_AUTH_JWT_PRIVATE_KEY`
    ///
    /// `*.pem` file location for the JWT private key.
    ///
    /// It defaults to `jwt.private.pem` if it does not exists.
    #[doku(as = "String", example = "jwt/private.pem")]
    #[config(as_type = "Option<PathBuf>")]
    pub private_key_file: PathBuf,
}

#[derive(Debug, Error)]
#[error("Could not load configuration for JWT authentication")]
pub struct JwtLoadError;

impl Jwt {
    pub(crate) fn from_partial(partial: PartialJwt) -> Result<Self, JwtLoadError> {
        let private_key_file = partial
            .private_key_file
            .unwrap_or_else(|| PathBuf::from("jwt.private.pem"));

        Ok(Self { private_key_file })
    }
}

impl PartialJwt {
    pub(crate) fn from_env() -> Result<Self, JwtLoadError> {
        let private_key_file = env::var_opt_parsed::<PathBuf>(vars::AUTH_JWT_PRIVATE_KEY)
            .change_context(JwtLoadError)?;

        Ok(Self { private_key_file })
    }
}
