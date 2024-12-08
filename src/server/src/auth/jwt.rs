use capwat_error::{ext::ResultExt, ApiErrorCategory, Error, Result};
use capwat_model::User;
use chrono::{TimeDelta, Utc};
use jsonwebtoken::{errors::ErrorKind, Algorithm, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::LazyLock;
use thiserror::Error;

use crate::App;

static JWT_HEADER: LazyLock<Header> = LazyLock::new(|| Header::new(Algorithm::RS256));
static JWT_LOGIN_ISSUER: &'static str = "capwat.api.login";

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginClaims {
    pub nbf: i64,
    pub exp: i64,
    pub iss: String,
    pub sub: i64,

    pub name: String,
    pub scope: Vec<String>,
}

#[derive(Debug, Error)]
#[error("Failed to decode as JWT")]
pub struct DecodeJwtError;

fn decode_jwt<T: DeserializeOwned>(
    app: &App,
    token: &str,
    issuer: String,
) -> Result<T, DecodeJwtError> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.leeway = 30;
    validation.validate_exp = true;
    validation.validate_nbf = true;
    validation.set_issuer(&[issuer]);

    let token = token.replace(char::is_whitespace, "");
    match jsonwebtoken::decode(&token, &app.jwt_decode_key, &validation) {
        Ok(d) => Ok(d.claims),
        Err(error) => match *error.kind() {
            ErrorKind::InvalidToken => {
                Err(Error::new(ApiErrorCategory::AccessDenied, DecodeJwtError))
            }
            ErrorKind::InvalidIssuer => {
                Err(Error::new(ApiErrorCategory::AccessDenied, DecodeJwtError))
            }
            ErrorKind::ExpiredSignature => {
                Err(Error::new(ApiErrorCategory::ExpiredToken, DecodeJwtError))
            }
            _ => Err(Error::unknown_generic(error).change_context(DecodeJwtError)),
        },
    }
}

#[derive(Debug, Error)]
#[error("Failed to encode as JWT")]
pub struct EncodeJwtError;

impl LoginClaims {
    pub fn decode(app: &App, token: &str) -> Result<Self, DecodeJwtError> {
        decode_jwt(app, token, JWT_LOGIN_ISSUER.to_string())
    }

    pub fn encode(&self, app: &App) -> Result<String, EncodeJwtError> {
        jsonwebtoken::encode(&JWT_HEADER, self, &app.jwt_encode_key)
            .change_context(EncodeJwtError)
            .attach_printable("could not encode login jwt claims")
    }

    pub fn generate(user: &User, scopes: &'static [&'static str]) -> LoginClaims {
        let now = Utc::now();
        Self {
            nbf: now.timestamp(),
            exp: (now + TimeDelta::days(1)).timestamp(),
            iss: JWT_LOGIN_ISSUER.to_string(),
            sub: user.id.0,

            name: user.name.clone(),
            scope: scopes
                .into_iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>(),
        }
    }
}
