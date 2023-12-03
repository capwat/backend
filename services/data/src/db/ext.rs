use error_stack::Report;

use super::pool::PoolError;

pub trait SqlxErrorExt<T> {
  fn into_db_error(self) -> error_stack::Result<T, PoolError>;
}

impl<T> SqlxErrorExt<T> for Result<T, sqlx::Error> {
  fn into_db_error(self) -> error_stack::Result<T, PoolError> {
    self.map_err(|e| match &e {
      sqlx::Error::Database(err)
        if err.message().ends_with("read-only transaction") =>
      {
        Report::new(e).change_context(PoolError::Readonly)
      },
      _ => Report::new(e).change_context(PoolError::Internal),
    })
  }
}

pub trait DbErrorExt2 {
  fn is_unhealthy(&self) -> bool;
  fn is_readonly(&self) -> bool;
}

impl<C> DbErrorExt2 for error_stack::Report<C> {
  fn is_unhealthy(&self) -> bool {
    self
      .downcast_ref::<PoolError>()
      .map(|v| matches!(v, PoolError::UnhealthyPool))
      .unwrap_or_default()
  }

  fn is_readonly(&self) -> bool {
    self
      .downcast_ref::<PoolError>()
      .map(|v| matches!(v, PoolError::Readonly))
      .unwrap_or_default()
  }
}
