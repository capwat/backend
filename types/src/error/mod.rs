use serde::{Deserialize, Serialize};
use serde_value::Value;
use std::fmt::Display;

mod codes;
mod serialization;

pub use codes::ErrorCode;

#[derive(Debug)]
pub enum ErrorType {
  Internal,
  ReadonlyMode,
  NotAuthenticated,
  Unknown(RawError),
}

impl ErrorType {
  #[must_use]
  pub const fn code(&self) -> ErrorCode {
    match self {
      ErrorType::Internal => ErrorCode::Internal,
      ErrorType::ReadonlyMode => ErrorCode::ReadonlyMode,
      ErrorType::NotAuthenticated => ErrorCode::NotAuthenticated,
      ErrorType::Unknown(n) => n.code,
    }
  }

  #[must_use]
  pub const fn subcode(&self) -> Option<u32> {
    match self {
      ErrorType::Unknown(data) => data.subcode,
      _ => None,
    }
  }

  pub fn message(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ErrorType::Internal => {
        f.write_str("Internal server occurred. Please try again later.")
      },
      ErrorType::ReadonlyMode => f.write_str(
        "This service is currently in read only mode. Please try again later.",
      ),
      ErrorType::NotAuthenticated => f.write_str("Not authenticated"),
      ErrorType::Unknown(d) => d.message.fmt(f),
    }
  }
}

impl Display for ErrorType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ErrorType::Internal => f.write_str("Failed to perform request"),
      ErrorType::ReadonlyMode => {
        f.write_str("Attempt to write to a read only database")
      },
      ErrorType::NotAuthenticated => {
        f.write_str("Attempt to access resource while not authenticated")
      },
      ErrorType::Unknown(n) => {
        write!(f, "({}", n.code)?;
        if let Some(subcode) = n.subcode {
          write!(f, ":{subcode}")?;
        }
        write!(f, "): {}", n.message)
      },
    }
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawError {
  pub code: ErrorCode,
  pub subcode: Option<u32>,
  pub message: String,
  // TODO: BTreeMap is very inefficient, we need to find a way
  //       on how to improve the performance of this.
  pub data: Option<Value>,
}
