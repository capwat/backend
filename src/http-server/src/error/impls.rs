use super::Error;

use actix_web::{body::BoxBody, http::StatusCode, HttpResponse};
use error_stack::Report;
use whim_types::Error as ErrorType;

impl actix_web::ResponseError for Error {
  fn status_code(&self) -> StatusCode {
    match self.error_type {
      ErrorType::Internal => StatusCode::INTERNAL_SERVER_ERROR,
      ErrorType::ReadonlyMode => StatusCode::SERVICE_UNAVAILABLE,
      ErrorType::InvalidFormBody(..) => StatusCode::BAD_REQUEST,
    }
  }

  fn error_response(&self) -> HttpResponse<BoxBody> {
    tracing::warn!(error = %self, "Error");
    HttpResponse::build(self.status_code()).json(&self.error_type)
  }
}

impl From<Report<whim_database::Error>> for Error {
  fn from(value: Report<whim_database::Error>) -> Self {
    match value.current_context() {
      whim_database::Error::Readonly => Error::from_report(ErrorType::ReadonlyMode, value),
      _ => Error::from_report(ErrorType::Internal, value),
    }
  }
}

impl From<validator::ValidateError> for Error {
  fn from(value: validator::ValidateError) -> Self {
    #[derive(Debug, thiserror::Error)]
    #[error("Validation error occurred")]
    struct ValidateError;
    Error::from_context(ErrorType::InvalidFormBody(value), ValidateError)
  }
}
