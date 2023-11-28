use std::fmt::{Debug, Display};
use thiserror::Error;

mod ignored;
mod serialization;
mod unknown;

pub mod codes;

pub use ignored::Ignored;
pub use unknown::Unknown;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ErrorType<T: ErrorCategory> {
  #[error("Internal server occurred. Please try again later.")]
  Internal,
  #[error(
    "This service is currently in read only mode. Please try again later."
  )]
  Readonly,
  #[error("You do not have permission to access this information")]
  Unauthorized,
  // This is for clients only actually but we're going to keep
  // this for federation purposes.
  #[error("{0}")]
  Unknown(Unknown),
  #[error(transparent)]
  Specific(#[from] T),
}

impl<T: ErrorCategory> ErrorType<T> {
  #[must_use]
  pub fn code(&self) -> u32 {
    match self {
      Self::Internal => codes::INTERNAL,
      Self::Readonly => codes::READONLY_MODE,
      Self::Unauthorized => codes::UNAUTHORIZED,

      Self::Unknown(n) => n.code,
      Self::Specific(..) => T::code(),
    }
  }

  #[must_use]
  pub fn subcode(&self) -> Option<u32> {
    match self {
      Self::Specific(n) => n.subcode(),
      Self::Unknown(n) => n.subcode,
      _ => None,
    }
  }

  #[cfg(feature = "server_impl")]
  pub fn server_message(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    match self {
      ErrorType::Internal => f.write_str("Failed to perform request"),
      ErrorType::Readonly => {
        f.write_str("Attempt to write to a read only database")
      },
      ErrorType::Unauthorized => {
        f.write_str("Attempt to access restricted resource")
      },
      ErrorType::Unknown(data) => {
        write!(f, "Received unknown error ({}", data.code)?;
        if let Some(subcode) = data.subcode {
          write!(f, ":{subcode}")?;
        }
        write!(f, ") - {}", data.message)
      },
      ErrorType::Specific(n) => n.server_message(f),
    }
  }

  fn message(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Unknown(n) => std::fmt::Display::fmt(&n, f),
      Self::Specific(n) => n.message(f),
      _ => std::fmt::Display::fmt(&self, f),
    }
  }

  fn needs_data_serialization(&self) -> bool {
    match self {
      Self::Unknown(n) => n.data.is_some(),
      Self::Specific(n) => n.needs_data_serialization(),
      _ => false,
    }
  }

  fn serialize_data<S>(
    &self,
    serializer: S,
  ) -> std::result::Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      Self::Specific(n) => n.serialize_data(serializer),
      Self::Unknown(n) => {
        <_ as serde::Serialize>::serialize(&n.data, serializer)
      },
      _ => unreachable!(),
    }
  }
}

pub trait ErrorCategory: Debug + Display + PartialEq + Eq {
  fn code() -> u32;
  fn subcode(&self) -> Option<u32>;

  #[cfg(feature = "server_impl")]
  #[doc(hidden)]
  fn server_message(&self, f: &mut std::fmt::Formatter<'_>)
    -> std::fmt::Result;

  #[doc(hidden)]
  fn message(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

  #[doc(hidden)]
  fn needs_data_serialization(&self) -> bool;

  #[doc(hidden)]
  fn deserialize_data<'de, D>(
    subcode: Option<u32>,
    deserializer: D,
  ) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
    Self: Sized;

  #[doc(hidden)]
  fn serialize_data<S>(
    &self,
    _serializer: S,
  ) -> std::result::Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    Err(serde::ser::Error::custom("Missing data"))
  }
}

// #[cfg(test)]
// mod tests {
//   use super::*;
//   use serde_test::Token;

//   #[track_caller]
//   fn assert_unit_variant(value: &Error, variant: &'static str) {
//     serde_test::assert_tokens(
//       value,
//       &[
//         Token::Struct { name: "Error", len: 1 },
//         Token::Str("type"),
//         Token::Str(variant),
//         Token::StructEnd,
//       ],
//     );
//   }

//   #[test]
//   fn test_serde_impl() {
//     assert_unit_variant(&Error::Internal, "internal");
//     assert_unit_variant(&Error::ReadonlyMode, "readonly_mode");
//   }
// }
