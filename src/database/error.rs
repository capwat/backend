use error_stack::Report;
use thiserror::Error;

/// Database related errors
#[derive(Debug, Error)]
pub enum Error {
  /// An error caused by an invalid Postgres connection
  /// url for either the primary or the replica pool.
  #[error("invalid connection url")]
  InvalidUrl,
  /// An error caused by an [`sqlx`] error.
  #[error("received a pool error: {0}")]
  Internal(sqlx::Error),
  /// The database pool (primary) is currently in read mode
  /// (most likely due to maintenance) and should not perform
  /// any writes.
  #[error("database is currently in read mode")]
  Readonly,
  /// Either the primary or replica database pools do not
  /// have reliable connection to transact to the database.
  #[error("unhealthy database pool")]
  UnhealthyPool,
}

/// Converts from a generic [sqlx] result into a [database compatible error](Error).
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

/// Lazily typed [`std::result::Result`] but the error generic
/// is filled up with [a database error](Error).
pub type Result<T> = error_stack::Result<T, Error>;

/// This trait deals with `error_stack::Report<Error>` because it is
/// annoying to implement code if [`Error`] is variant of something:
///
/// ```rust,ignore
/// let result = db.do_query(...);
/// if let Err(e) = result {
///   let is_unhealthy = e.downcast_ref::<whim_db_core::Error>()
///     .map(|v| matches!(v, whim_database::Error::UnhealthyPool))
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
