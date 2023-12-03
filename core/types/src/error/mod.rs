use serde_value::Value;
use std::fmt::Display;

mod code;
mod consts;
mod serialize;
mod variants;

pub use code::ErrorCode;
pub use variants::*;

/// Possible error outcomes in Capwat API.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
  Internal,
  ReadonlyMode,
  NotAuthenticated,
  InvalidFormBody(Box<InvalidFormBody>),
  LoginUser(Box<LoginUser>),
  Unknown(Box<Unknown>),
}

impl Error {
  #[must_use]
  pub const fn code(&self) -> ErrorCode {
    match self {
      Self::Internal => ErrorCode::Internal,
      Self::ReadonlyMode => ErrorCode::ReadonlyMode,
      Self::NotAuthenticated => ErrorCode::NotAuthenticated,
      Self::InvalidFormBody(..) => ErrorCode::InvalidFormBody,
      Self::LoginUser(..) => ErrorCode::LoginUser,
      Self::Unknown(data) => data.code,
    }
  }

  #[must_use]
  pub fn message(&self) -> &str {
    match self {
      Self::Internal => consts::INTERNAL_MSG,
      Self::ReadonlyMode => consts::READONLY_MODE_MSG,
      Self::NotAuthenticated => consts::NOT_AUTHENTICATED_MSG,
      Self::InvalidFormBody(..) => consts::INVALID_FORM_BODY_MSG,
      Self::LoginUser(data) => match data.as_ref() {
        LoginUser::InvalidCredientials => {
          consts::login_user::INVALID_CREDIENTIALS_MSG
        },
        LoginUser::Banned { .. } => consts::login_user::BANNED_MSG,
      },
      Self::Unknown(d) => &d.message,
    }
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Internal => f.write_str("Failed to perform request"),
      Self::ReadonlyMode => {
        f.write_str("Attempt to write to a read only database")
      },
      Self::NotAuthenticated => {
        f.write_str("Attempt to access resource while not authenticated")
      },
      Self::InvalidFormBody(..) => f.write_str("Sent an invalid form body"),
      Self::LoginUser(data) => match data.as_ref() {
        LoginUser::InvalidCredientials => {
          f.write_str("User put invalid credentials")
        },
        LoginUser::Banned { .. } => {
          f.write_str("Attempt to log in a banned user")
        },
      },
      Self::Unknown(n) => {
        write!(f, "({}", n.code)?;
        if let Some(subcode) = n.subcode {
          write!(f, ":{subcode}")?;
        }
        write!(f, "): {}", n.message)
      },
    }
  }
}

#[derive(Debug)]
pub struct Unknown {
  pub code: ErrorCode,
  pub subcode: Option<u32>,
  pub message: String,
  pub data: Option<Value>,
}

impl PartialEq for Unknown {
  fn eq(&self, other: &Self) -> bool {
    // We're comparing with code, subcode and data fields only
    if !self.code.eq(&other.code) {
      return false;
    }

    if !self.subcode.eq(&other.subcode) {
      return false;
    }

    self.data.eq(&other.data)
  }
}

impl Eq for Unknown {}

#[cfg(test)]
mod tests {
  use super::Error;
  use serde::{Deserialize, Serialize};
  use static_assertions::assert_impl_all;
  use std::fmt::{Debug, Display};

  assert_impl_all!(Error: Debug, Display, PartialEq, Eq, Deserialize<'static>, Serialize);
}
