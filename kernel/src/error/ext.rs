use capwat_types::error::{ErrorCategory, ErrorType, Ignored};
use error_stack::Context;

use super::{Error, Result};

pub trait StdContext<Err: ErrorCategory = Ignored> {
  type Ok;

  fn with_capwat_error<Category: ErrorCategory>(
    self,
    error_type: ErrorType<Category>,
  ) -> Result<Self::Ok, Category>;

  fn into_capwat_error(self) -> Result<Self::Ok, Err>;
}

pub trait ErrorStackContext<Err: ErrorCategory = Ignored> {
  type Ok;

  fn with_capwat_error<Category: ErrorCategory>(
    self,
    error_type: ErrorType<Category>,
  ) -> Result<Self::Ok, Category>;

  fn into_capwat_error(self) -> Result<Self::Ok, Err>;
}

impl<T, E: ErrorCategory, C: Context> StdContext<E>
  for std::result::Result<T, C>
{
  type Ok = T;

  fn with_capwat_error<Category: ErrorCategory>(
    self,
    error_type: ErrorType<Category>,
  ) -> Result<T, Category> {
    self.map_err(|e| Error::from_context(error_type, e))
  }

  fn into_capwat_error(self) -> Result<T, E> {
    self.map_err(|e| Error::from_context(ErrorType::Internal, e))
  }
}

impl<T, E: ErrorCategory, C: Context> ErrorStackContext<E>
  for error_stack::Result<T, C>
{
  type Ok = T;

  fn with_capwat_error<Category: ErrorCategory>(
    self,
    error_type: ErrorType<Category>,
  ) -> Result<T, Category> {
    self.map_err(|e| Error::from_report(error_type, e))
  }

  fn into_capwat_error(self) -> Result<T, E> {
    self.map_err(|e| Error::from_report(ErrorType::Internal, e))
  }
}
