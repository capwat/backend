use super::{Error, ErrorType, Result};

use error_stack::Context;
use std::result::Result as StdResult;
use whim_types::error::server::ServerError;

pub trait AppContext<T> {
  fn change_context(self, context: impl Context) -> Result<T>;
  fn change_err_type(self, error_type: impl ErrorType + 'static) -> Result<T>;
}

pub trait StdContext<T> {
  fn change_err_type(self, error_type: impl ErrorType + 'static) -> Result<T>;
  fn into_http_result(self) -> Result<T>;
}

pub trait ErrorStackContext<T> {
  fn change_type(self, error_type: impl ErrorType + 'static) -> Result<T>;
  fn into_http_result(self) -> Result<T>;
}

impl<T> AppContext<T> for Result<T> {
  fn change_context(self, context: impl Context) -> Result<T> {
    self.map_err(|e| e.change_context(context))
  }

  fn change_err_type(self, error_type: impl ErrorType + 'static) -> Result<T> {
    self.map_err(|e| e.change_type(error_type))
  }
}

impl<T, C: Context> StdContext<T> for StdResult<T, C> {
  fn change_err_type(self, error_type: impl ErrorType + 'static) -> Result<T> {
    self.map_err(|e| Error::from_context(error_type, e))
  }

  fn into_http_result(self) -> Result<T> {
    self.map_err(|e| Error::from_context(ServerError::Internal, e))
  }
}

impl<T, C: Context> ErrorStackContext<T> for error_stack::Result<T, C> {
  fn change_type(self, error_type: impl ErrorType + 'static) -> Result<T> {
    self.map_err(|e| Error::from_report(error_type, e))
  }

  fn into_http_result(self) -> Result<T> {
    self.map_err(|e| Error::from_report(ServerError::Internal, e))
  }
}
