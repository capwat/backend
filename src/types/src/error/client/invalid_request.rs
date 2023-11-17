use std::fmt::Display;

use crate::error::{codes, Tertiary};
use whim_derives::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[error(code = "codes::INVALID_REQUEST")]
pub enum InvalidRequest {
  #[error(subcode = "codes::invalid_request::INVALID_FORM_BODY")]
  InvalidFormBody(validator::ValidateError),
  #[error(
    subcode = "codes::invalid_request::UNSUPPORTED_API_VERSION",
    message = "Your specified API version is not currently supported"
  )]
  UnsupportedApiVersion,
}

impl Tertiary for validator::ValidateError {
  fn message(&self) -> std::borrow::Cow<'static, str> {
    std::borrow::Cow::Owned(self.to_string())
  }
}

impl Display for InvalidRequest {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Client sent an invalid request")
  }
}
