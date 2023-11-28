use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorCode {
  Internal,
  ReadonlyMode,
  NotAuthenticated,
  Unknown(u32),
}

impl Display for ErrorCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_u64().fmt(f)
  }
}

const INTERNAL: u32 = 1;
const READONLY_MODE: u32 = 2;
const NOT_AUTHENTICATED: u32 = 3;

impl ErrorCode {
  #[must_use]
  pub const fn as_u64(self) -> u32 {
    match self {
      Self::Internal => INTERNAL,
      Self::ReadonlyMode => READONLY_MODE,
      Self::NotAuthenticated => NOT_AUTHENTICATED,
      Self::Unknown(value) => value,
    }
  }

  #[must_use]
  pub const fn from_code(value: u32) -> Self {
    match value {
      INTERNAL => Self::Internal,
      READONLY_MODE => Self::ReadonlyMode,
      NOT_AUTHENTICATED => Self::NotAuthenticated,
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
    val.as_u64()
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
    self.as_u64().serialize(serializer)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_test::Token;

  fn test_variant(code: ErrorCode) {
    serde_test::assert_tokens(&code, &[Token::U32(code.as_u64())]);
  }

  #[test]
  fn test_serde_impl() {
    test_variant(ErrorCode::Internal);
    test_variant(ErrorCode::ReadonlyMode);
    test_variant(ErrorCode::NotAuthenticated);
    test_variant(ErrorCode::Unknown(u32::MAX));
  }
}
