use capwat_error::ext::ResultExt;
use capwat_error::Result;
use capwat_error::{ext::NoContextResultExt, Error};
use capwat_macros::ConfigParts;
use capwat_utils::{env, ProtectedString};
use doku::Document;
use serde::Deserialize;
use thiserror::Error;

use crate::vars;

#[derive(Debug, Document, ConfigParts)]
#[config(attr(derive(Debug, Deserialize)))]
#[config(attr(serde(rename_all = "kebab-case")))]
pub struct HCaptcha {
    /// **Environment variables**:
    /// - `CAPWAT_HCAPTCHA_TOKEN`
    /// - `HCAPTCHA_TOKEN`
    ///
    /// Your hCaptcha secret token.
    #[doku(as = "String", example = "INSERT_YOUR_HCAPTCHA_TOKEN_HERE")]
    pub secret_token: ProtectedString,
}

impl HCaptcha {
    pub(crate) fn from_partial(partial: PartialHCaptcha) -> Result<Self, HCaptchaLoadError> {
        let token = partial
            .secret_token
            .ok_or_else(|| Error::unknown(HCaptchaLoadError))
            .attach_printable_lazy(|| {
                format!(
                    "{} is required to setup for hCaptcha integration",
                    vars::HCAPTCHA_SECRET_TOKEN
                )
            })?;

        Ok(Self {
            secret_token: token,
        })
    }
}

#[derive(Debug, Error)]
#[error("Could not load configuration for hCaptcha")]
pub struct HCaptchaLoadError;

impl PartialHCaptcha {
    pub fn from_env() -> Result<Option<Self>, HCaptchaLoadError> {
        let secret_token = env::var_opt(vars::HCAPTCHA_SECRET_TOKEN2)
            .map(Some)
            .transpose()
            .unwrap_or_else(|| env::var_opt(vars::HCAPTCHA_SECRET_TOKEN))
            .change_context(HCaptchaLoadError)?;

        Ok(secret_token.map(|token| Self {
            secret_token: Some(ProtectedString::new(token)),
        }))
    }
}
