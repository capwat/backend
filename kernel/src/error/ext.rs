use capwat_types::error::ErrorType;
use error_stack::Context;

use super::{Error, Result};

pub trait StdContext {
  type Ok;

  fn with_capwat(self, error_type: ErrorType) -> Result<Self::Ok>;
  fn into_capwat(self) -> Result<Self::Ok>;
}

pub trait ErrorStackContext {
  type Ok;

  fn with_capwat_error(self, error_type: ErrorType) -> Result<Self::Ok>;
  fn into_capwat_error(self) -> Result<Self::Ok>;
}

impl<T, C: Context> StdContext for std::result::Result<T, C> {
  type Ok = T;

  fn with_capwat(self, error_type: ErrorType) -> Result<T> {
    self.map_err(|e| Error::from_context(error_type, e))
  }

  fn into_capwat(self) -> Result<T> {
    self.map_err(|e| Error::from_context(ErrorType::Internal, e))
  }
}

impl<T, C: Context> ErrorStackContext for error_stack::Result<T, C> {
  type Ok = T;

  fn with_capwat_error(self, error_type: ErrorType) -> Result<T> {
    self.map_err(|e| Error::from_report(error_type, e))
  }

  fn into_capwat_error(self) -> Result<T> {
    self.map_err(|e| Error::from_report(ErrorType::Internal, e))
  }
}
