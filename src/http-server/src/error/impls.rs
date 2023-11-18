use super::Error;
use actix_web::{body::BoxBody, HttpResponse};

impl actix_web::ResponseError for Error {
  fn status_code(&self) -> actix_web::http::StatusCode {
    self.error_type.status_code()
  }

  fn error_response(&self) -> HttpResponse<BoxBody> {
    match self.error_type.response() {
      Ok(n) => n,
      Err(e) => e.error_response(),
    }
  }
}
