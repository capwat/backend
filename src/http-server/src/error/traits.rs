use actix_web::{
  http::{header, StatusCode},
  HttpResponse, HttpResponseBuilder,
};
use serde::Serialize;
use whim_types::error::{self, server::ServerError, Primary, Serializable};

use super::{ext::StdContext, Error, Result};

trait ToJson {
  fn from_json(&mut self, json: impl Serialize) -> Result<HttpResponse>;
}

impl ToJson for HttpResponseBuilder {
  fn from_json(&mut self, json: impl Serialize) -> Result<HttpResponse> {
    let body = serde_json::to_string(&json).into_http_result()?;
    let resp = self
      .insert_header((header::CONTENT_TYPE, mime::APPLICATION_JSON))
      .body(body);

    Ok(resp)
  }
}

pub trait ErrorType: Primary {
  fn status_code(&self) -> StatusCode;
  fn response(&self) -> Result<HttpResponse>;
}

impl ErrorType for ServerError {
  fn status_code(&self) -> StatusCode {
    match self {
      Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
      Self::ReadonlyMode => StatusCode::SERVICE_UNAVAILABLE,
    }
  }

  fn response(&self) -> Result<HttpResponse> {
    let mut resp = HttpResponseBuilder::new(self.status_code());
    let body = Serializable::new(self);
    match self {
      Self::Internal => Ok(
        resp
          .from_json(body)
          .expect("failed to parse internal error"),
      ),
      Self::ReadonlyMode => HttpResponse::ServiceUnavailable().from_json(body),
    }
  }
}

impl ErrorType for error::client::InvalidRequest {
  fn status_code(&self) -> StatusCode {
    match self {
      Self::InvalidFormBody(..) => StatusCode::BAD_REQUEST,
      Self::UnsupportedApiVersion => StatusCode::BAD_REQUEST,
    }
  }

  fn response(&self) -> Result<HttpResponse> {
    let body = Serializable::new(self);
    HttpResponseBuilder::new(self.status_code()).from_json(body)
  }
}

impl ErrorType for error::client::LoginUser {
  fn status_code(&self) -> StatusCode {
    match self {
      Self::InvalidCredentials => StatusCode::FORBIDDEN,
    }
  }

  fn response(&self) -> Result<HttpResponse> {
    let body = Serializable::new(self);
    HttpResponseBuilder::new(self.status_code()).from_json(body)
  }
}

impl ErrorType for error::client::RegisterUser {
  fn status_code(&self) -> StatusCode {
    match self {
      Self::Closed => StatusCode::LOCKED,
      Self::EmailExists => StatusCode::FORBIDDEN,
      Self::EmailRequired => StatusCode::BAD_REQUEST,
      Self::UserExists => StatusCode::FORBIDDEN,
    }
  }

  fn response(&self) -> Result<HttpResponse> {
    let body = Serializable::new(self);
    match self {
      Self::Closed => HttpResponse::Locked().from_json(body),
      Self::EmailExists => HttpResponse::Forbidden().from_json(body),
      Self::EmailRequired => HttpResponse::BadRequest().from_json(body),
      Self::UserExists => HttpResponse::Forbidden().from_json(body),
    }
  }
}

impl From<whim_database::Error> for Error {
  fn from(value: whim_database::Error) -> Self {
    match &value {
      whim_database::Error::Readonly => Error::from_context(ServerError::ReadonlyMode, value),
      whim_database::Error::UnhealthyPool => Error::from_context(ServerError::Internal, value),
      _ => Error::from_context(ServerError::Internal, value),
    }
  }
}

impl From<validator::ValidateError> for Error {
  fn from(value: validator::ValidateError) -> Self {
    #[derive(Debug, thiserror::Error)]
    #[error("Validation error occurred")]
    struct ValidateError;
    Error::from_context(
      error::client::InvalidRequest::InvalidFormBody(value),
      ValidateError,
    )
  }
}
