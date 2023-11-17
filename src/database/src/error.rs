use error_stack::Report;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
  #[error("invalid connection url")]
  InvalidUrl,
  #[error("received a pool error")]
  Internal(sqlx::Error),
  #[error("database is currently in read mode")]
  Readonly,
  #[error("unhealthy database pool")]
  UnhealthyPool,
}

pub trait ErrorExt<T> {
  fn into_db_error(self) -> Result<T>;
}

impl<T> ErrorExt<T> for std::result::Result<T, sqlx::Error> {
  fn into_db_error(self) -> Result<T> {
    self.map_err(|e| match &e {
      sqlx::Error::Database(err) if err.message().ends_with("read-only transaction") => {
        Report::new(e).change_context(Error::Readonly)
      }
      _ => Report::new(Error::Internal(e)),
    })
  }
}

pub type Result<T> = error_stack::Result<T, Error>;

/// This trait deals with `error_stack::Report<Error>` because it is
/// annoying to implement code if [`Error`] is variant of something:
///
/// ```no-run,rs
/// let result = db.do_query(...);
/// if let Err(e) = result {
///   let is_unhealthy = e.downcast_ref::<whim_db_core::Error>()
///     .map(|v| matches!(v, whim_db_core::UnhealthyPool))
///     .unwrap_or_default();
///   ...
/// }
/// ```
pub trait ErrorExt2 {
  fn is_unhealthy(&self) -> bool;
  fn is_readonly(&self) -> bool;
}

impl ErrorExt2 for error_stack::Report<Error> {
  fn is_unhealthy(&self) -> bool {
    self
      .downcast_ref::<Error>()
      .map(|v| matches!(v, Error::UnhealthyPool))
      .unwrap_or_default()
  }

  fn is_readonly(&self) -> bool {
    self
      .downcast_ref::<Error>()
      .map(|v| matches!(v, Error::Readonly))
      .unwrap_or_default()
  }
}
