use std::fmt::Display;

use crate::error::codes;
use whim_derives::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error(code = "codes::REGISTER_USER")]
pub enum RegisterUser {
  #[error(
    subcode = "codes::register_user::CLOSED",
    message = "User registration is closed by a site administrator"
  )]
  Closed,
  #[error(
    subcode = "codes::register_user::EMAIL_EXISTS",
    message = "This email already exists"
  )]
  EmailExists,
  #[error(
    subcode = "codes::register_user::EMAIL_REQUIRED",
    message = "Email is required for site registration"
  )]
  EmailRequired,
  #[error(
    subcode = "codes::register_user::USER_EXISTS",
    message = "User info specified already exists"
  )]
  UserExists,
}

impl Display for RegisterUser {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Failed to register user")
  }
}
