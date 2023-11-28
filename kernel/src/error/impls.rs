use capwat_types::error::ErrorType;
use error_stack::Report;

use super::Error;
use crate::db;

impl From<Report<db::PoolError>> for Error {
  fn from(value: Report<db::PoolError>) -> Self {
    // SAFETY: already defined that is a pool error
    unsafe {
      match value.current_context() {
        db::PoolError::Readonly => {
          Error::from_report(ErrorType::ReadonlyMode, value)
        },
        _ => Error::from_report(ErrorType::Internal, value),
      }
    }
  }
}

impl From<tonic::Status> for Error {
  fn from(value: tonic::Status) -> Self {
    Error::from_tonic(&value)
  }
}

// impl From<validator::ValidateError> for Error {
//   fn from(value: validator::ValidateError) -> Self {
//     #[derive(Debug, thiserror::Error)]
//     #[error("Validation error occurred")]
//     struct ValidateError;
//     Error::from_context(ErrorType::InvalidFormBody(value), ValidateError)
//   }
// }
