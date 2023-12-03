use std::num::NonZeroU64;
use thiserror::Error;

use crate::{id::marker::Marker, Id};

impl<'q, T: Marker> sqlx::Encode<'q, sqlx::Postgres> for Id<T> {
  // already checked
  #[allow(clippy::cast_sign_loss)]
  fn encode_by_ref(
    &self,
    buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
  ) -> sqlx::encode::IsNull {
    const I64_AS_U64_MAX: u64 = i64::MAX as u64;

    // Already checked
    #[allow(clippy::cast_possible_wrap)]
    if self.value.get() > I64_AS_U64_MAX {
      sqlx::encode::IsNull::Yes
    } else {
      <i64 as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(
        &(self.value.get() as i64),
        buf,
      )
    }
  }
}

impl<'r, T: Marker> sqlx::Decode<'r, sqlx::Postgres> for Id<T> {
  // already checked
  #[allow(clippy::cast_sign_loss)]
  fn decode(
    value: <sqlx::Postgres as sqlx::database::HasValueRef<'r>>::ValueRef,
  ) -> Result<Self, sqlx::error::BoxDynError> {
    #[derive(Debug, Error)]
    #[error("all IDs must be positive")]
    struct NegativeIdError;

    #[derive(Debug, Error)]
    #[error("all IDs must not be equal to 0")]
    struct EqualToZeroError;

    let value = <i64 as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
    if value.is_negative() {
      Err(Box::new(NegativeIdError))
    } else if let Some(inner) = NonZeroU64::new(value as u64) {
      Ok(Id::from_nonzero(inner))
    } else {
      Err(Box::new(EqualToZeroError))
    }
  }
}

impl<T: Marker> sqlx::Type<sqlx::Postgres> for Id<T> {
  fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
    <i64 as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}

#[cfg(all(test, feature = "db-testing"))]
mod tests {
  use super::*;
  use crate::id::marker::AnyMarker;
  use static_assertions::assert_impl_all;

  assert_impl_all!(Id<AnyMarker>:
    sqlx::Decode<'static, sqlx::Postgres>,
    sqlx::Encode<'static, sqlx::Postgres>, sqlx::Type<sqlx::Postgres>
  );

  #[derive(Debug, PartialEq, Eq, sqlx::FromRow)]
  struct TestOutput {
    pub value: Id<AnyMarker>,
  }

  #[sqlx::test]
  async fn test_encode(pool: sqlx::PgPool) {
    let mut conn = pool.begin().await.unwrap();
    let id = Id::<AnyMarker>::new(123);
    sqlx::query("SELECT $1").bind(id).fetch_one(&mut *conn).await.unwrap();

    let id = Id::<AnyMarker>::new(u64::MAX);
    let result = sqlx::query_as::<_, TestOutput>("SELECT $1 as value")
      .bind(id)
      .fetch_one(&mut *conn)
      .await;

    let Err(sqlx::Error::ColumnDecode { source, .. }) = result else {
      panic!("expected column error")
    };
    assert!(source
      .downcast_ref::<sqlx::error::UnexpectedNullError>()
      .is_some());
  }

  #[sqlx::test]
  async fn test_decode(pool: sqlx::PgPool) {
    let mut conn = pool.begin().await.unwrap();
    let result = sqlx::query_as::<_, TestOutput>("SELECT 1234::int8 as value")
      .fetch_one(&mut *conn)
      .await;

    assert_eq!(result.unwrap().value, 1234u64);

    let result = sqlx::query_as::<_, TestOutput>("SELECT -1234::int8 as value")
      .fetch_one(&mut *conn)
      .await;

    let Err(sqlx::Error::ColumnDecode { source, .. }) = result else {
      panic!("expected column error")
    };
    assert_eq!(source.to_string(), "all IDs must be positive");

    let result = sqlx::query_as::<_, TestOutput>("SELECT 0::int8 as value")
      .fetch_one(&mut *conn)
      .await;

    let Err(sqlx::Error::ColumnDecode { source, .. }) = result else {
      panic!("expected column error")
    };
    assert_eq!(source.to_string(), "all IDs must not be equal to 0");
  }
}
