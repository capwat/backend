use error_stack::{Context, Report};
use thiserror::Error;
use tracing_error::SpanTrace;

#[derive(Debug, Error)]
pub enum ErrorType<T> {
  #[error("Internal server error occurred")]
  Internal,
  #[error("Service unavailable")]
  Unavailable,
  #[error("This service is currently in read only mode")]
  Readonly,
  #[error(transparent)]
  Other(#[from] T),
}

pub struct Error<T> {
  error_type: ErrorType<T>,
  report: Option<Report<Box<dyn Context>>>,
  trace: SpanTrace,
}

// use thiserror::Error;

// #[derive(Debug, Error)]
// pub enum Error<T> {
//   #[error("Internal error occurred")]
//   Internal,
// }

// impl<T> Error<T> {}
