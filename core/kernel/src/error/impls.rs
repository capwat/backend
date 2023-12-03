use error_stack::{Context, Report};

use super::{
  ext::{ErrorExt, ErrorExt2},
  Error, ErrorCategory, ReportIntoError,
};

impl<T, C: Context> ErrorExt<T> for std::result::Result<T, C> {
  fn with_capwat_error(self, error_type: ErrorCategory) -> Result<T, Error> {
    self.map_err(|e| Error::from_context(error_type, e))
  }

  fn into_capwat_error(self) -> Result<T, Error> {
    self.map_err(|e| Error::from_context(ErrorCategory::Internal, e))
  }
}

impl<T, C: Context> ErrorExt2<T> for error_stack::Result<T, C> {
  fn with_capwat_error(self, error_type: ErrorCategory) -> Result<T, Error> {
    self.map_err(|e| Error::from_report(error_type, e))
  }

  fn into_capwat_error(self) -> Result<T, Error> {
    self.map_err(|e| Error::from_report(ErrorCategory::Internal, e))
  }
}

#[cfg(feature = "grpc")]
use super::IntoError;
#[cfg(feature = "grpc")]
use percent_encoding::NON_ALPHANUMERIC;
#[cfg(feature = "grpc")]
use thiserror::Error;

#[cfg(feature = "grpc")]
impl From<tonic::Status> for Error {
  fn from(value: tonic::Status) -> Self {
    Error::from_rpc(&value)
  }
}

#[cfg(feature = "grpc")]
impl From<Error> for tonic::Status {
  fn from(value: Error) -> Self {
    Error::into_rpc(&value)
  }
}

#[cfg(feature = "grpc")]
impl<T: IntoError> From<T> for Error {
  fn from(value: T) -> Self {
    value.into_capwat_error()
  }
}

// SAFETY: As long as the user confirms that it is not Report<()>
impl<T: ReportIntoError> From<Report<T>> for Error {
  fn from(value: Report<T>) -> Self {
    let error_type = unsafe { value.current_context().category() };
    Error::from_report(error_type, value)
  }
}

impl Error {
  #[must_use]
  #[cfg(feature = "grpc")]
  pub fn from_rpc(status: &tonic::Status) -> Self {
    #[derive(Debug, Error)]
    #[error("x-error-data metadata from gRPC transmission is invalid")]
    struct InvalidErrorData;

    if let Some(data) = status.metadata().get("x-error-data").and_then(|v| {
      percent_encoding::percent_decode(v.as_bytes()).decode_utf8().ok()
    }) {
      if let Ok(category) = serde_json::from_str(&data) {
        return Self::new(category);
      }
    }

    Self::new(ErrorCategory::Internal)
  }

  #[must_use]
  #[cfg(feature = "grpc")]
  pub fn into_rpc(&self) -> tonic::Status {
    use tonic::Code;

    let code = match self.category {
      ErrorCategory::ReadonlyMode => Code::Unavailable,
      ErrorCategory::NotAuthenticated => Code::Unauthenticated,
      ErrorCategory::InvalidFormBody(..) => Code::InvalidArgument,
      _ => Code::Internal,
    };

    let mut status = tonic::Status::new(code, self.category.message());
    let metadata = status.metadata_mut();

    // Keep the rest of the error data in JSON on a header
    //
    // This line below is very critical for serializing
    // internal errors if serialization with other error
    // types fails.
    if matches!(self.category, ErrorCategory::Internal) {
      let content = serde_json::to_string(&self.category)
        .expect("failed to serialize internal error struct");

      let content =
        percent_encoding::percent_encode(content.as_bytes(), NON_ALPHANUMERIC)
          .to_string();

      metadata.insert(
        "x-error-data",
        content.parse().expect("failed to parse encoded error data"),
      );

      return status;
    }

    match serde_json::to_string(&self.category) {
      Ok(data) => {
        let content =
          percent_encoding::percent_encode(data.as_bytes(), NON_ALPHANUMERIC)
            .to_string();

        match content.parse() {
          Ok(n) => {
            metadata.insert("x-error-data", n);
            status
          },
          Err(e) => Error::from_context(ErrorCategory::Internal, e).into_rpc(),
        }
      },
      Err(err) => Error::from_context(ErrorCategory::Internal, err).into_rpc(),
    }
  }
}
