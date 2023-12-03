use super::{Error, ErrorCategory};

pub trait ErrorExt<T> {
  fn with_capwat_error(self, error_type: ErrorCategory) -> Result<T, Error>;
  fn into_capwat_error(self) -> Result<T, Error>;
}

// This is for `error-stack` result types
pub trait ErrorExt2<T> {
  fn with_capwat_error(self, error_type: ErrorCategory) -> Result<T, Error>;
  fn into_capwat_error(self) -> Result<T, Error>;
}
