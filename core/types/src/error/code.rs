use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::consts;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorCode {
  Internal,
  ReadonlyMode,
  NotAuthenticated,
  InvalidFormBody,
  LoginUser,
  Unknown(u32),
}

impl Display for ErrorCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_u32().fmt(f)
  }
}

impl ErrorCode {
  #[must_use]
  pub const fn as_u32(self) -> u32 {
    match self {
      Self::Internal => consts::INTERNAL_CODE,
      Self::ReadonlyMode => consts::READONLY_MODE_CODE,
      Self::NotAuthenticated => consts::NOT_AUTHENTICATED_CODE,
      Self::InvalidFormBody => consts::INVALID_FORM_BODY_CODE,
      Self::LoginUser => consts::LOGIN_USER_CODE,
      Self::Unknown(value) => value,
    }
  }

  #[must_use]
  pub const fn from_code(value: u32) -> Self {
    match value {
      consts::INTERNAL_CODE => Self::Internal,
      consts::READONLY_MODE_CODE => Self::ReadonlyMode,
      consts::NOT_AUTHENTICATED_CODE => Self::NotAuthenticated,
      consts::INVALID_FORM_BODY_CODE => Self::InvalidFormBody,
      consts::LOGIN_USER_CODE => Self::LoginUser,
      _ => Self::Unknown(value),
    }
  }
}

impl From<u32> for ErrorCode {
  #[must_use]
  fn from(value: u32) -> Self {
    Self::from_code(value)
  }
}

impl From<ErrorCode> for u32 {
  #[must_use]
  fn from(val: ErrorCode) -> Self {
    val.as_u32()
  }
}

impl<'de> Deserialize<'de> for ErrorCode {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let value = u32::deserialize(deserializer)?;
    Ok(Self::from_code(value))
  }
}

impl Serialize for ErrorCode {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    self.as_u32().serialize(serializer)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_test::Token;

  const MAP: &[(ErrorCode, u32)] = &[
    (ErrorCode::Internal, 1),
    (ErrorCode::ReadonlyMode, 2),
    (ErrorCode::NotAuthenticated, 3),
    (ErrorCode::InvalidFormBody, 4),
    (ErrorCode::Unknown(100_000), 100_000),
  ];

  #[test]
  fn test_serde_impl() {
    for (variant, code) in MAP {
      serde_test::assert_tokens(variant, &[Token::U32(*code)]);
      assert_eq!(*code, variant.as_u32());
    }
  }
}
