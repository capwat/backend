// TODO: Implement future proof error structures that will be used in future API versions
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Error {
  Internal,
  InvalidFormBody(validator::ValidateError),
  ReadonlyMode,
}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Internal => f.write_str("Failed to perform request"),
      Error::InvalidFormBody(..) => f.write_str("User performed request with invalid body"),
      Error::ReadonlyMode => f.write_str("Attempt to write read-only database"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_test::Token;

  #[track_caller]
  fn assert_unit_variant(value: Error, variant: &'static str) {
    serde_test::assert_tokens(
      &value,
      &[
        Token::Struct {
          name: "Error",
          len: 1,
        },
        Token::Str("type"),
        Token::Str(variant),
        Token::StructEnd,
      ],
    );
  }

  #[test]
  fn test_serde_impl() {
    assert_unit_variant(Error::Internal, "internal");
    assert_unit_variant(Error::ReadonlyMode, "readonly_mode");
  }
}
