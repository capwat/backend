use crate::error::codes;
use std::fmt::Display;
use whim_derives::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error(code = "codes::LOGIN_USER")]
pub enum LoginUser {
  #[error(
    subcode = "codes::login_user::INVALID_CREDENTIALS",
    message = "Invalid credentials!"
  )]
  InvalidCredentials,
}

impl Display for LoginUser {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Failed to login user")
  }
}
