use once_cell::sync::Lazy;
use serde::de::{Error as DeError, Unexpected};
use std::{
  fmt::{Debug, Display},
  hash::Hash,
  marker::PhantomData,
  num::NonZeroU64,
};
use thiserror::Error;

use self::marker::Marker;
use crate::types::Timestamp;

pub mod marker;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id<T: Marker> {
  value: NonZeroU64,
  phantom: PhantomData<T>,
}

impl<T: Marker> Id<T> {
  /// # Panics
  ///
  /// It will panic if the value is 0.
  #[must_use]
  #[track_caller]
  pub const fn new(n: u64) -> Self {
    if let Some(id) = Self::new_checked(n) {
      id
    } else {
      panic!("value is zero")
    }
  }

  /// Creates an ID from [NonZeroU64] value.
  ///
  /// # Safety
  ///
  /// This function assumes the value is not equal to zero.
  #[must_use]
  pub const fn from_nonzero(n: NonZeroU64) -> Self {
    Self {
      value: n,
      phantom: PhantomData,
    }
  }

  #[must_use]
  pub const fn new_checked(n: u64) -> Option<Self> {
    if let Some(n) = NonZeroU64::new(n) {
      Some(Self::from_nonzero(n))
    } else {
      None
    }
  }

  #[must_use]
  pub const fn get(self) -> u64 {
    self.value.get()
  }

  #[must_use]
  pub const fn into_nonzero(self) -> NonZeroU64 {
    self.value
  }

  #[must_use]
  pub const fn cast<M: Marker>(self) -> Id<M> {
    Id {
      value: self.value,
      phantom: PhantomData,
    }
  }

  #[must_use]
  pub fn timestamp(self) -> Timestamp {
    Timestamp::from_snowflake(self.value)
  }
}

impl<T: Marker> Debug for Id<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // This is to improve performance but it will not be a significant boost because
    // we're going to use this for telemetry anyway. It's better than nothing.
    use heck::ToSnakeCase;
    static MARKER_MODULE: Lazy<String> = Lazy::new(|| {
      format!(
        "{}::types::id::marker::",
        env!("CARGO_PKG_NAME").to_snake_case()
      )
    });

    // This is to assume that all ID markers are defined in `marker` module
    let type_name = std::any::type_name::<T>();
    let type_name = if type_name.starts_with(&*MARKER_MODULE) {
      type_name.split("::").last().unwrap_or(type_name)
    } else {
      type_name
    };
    write!(f, "Id::<{type_name}>({})", self.value.get())
  }
}

impl<T: Marker> Display for Id<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    Display::fmt(&self.value.get(), f)
  }
}

impl<T: Marker> Hash for Id<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    state.write_u64(self.value.get());
  }
}

impl<'de, T: Marker> serde::Deserialize<'de> for Id<T> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    struct Visitor<T: Marker>(PhantomData<T>);

    impl<'de, T: Marker> serde::de::Visitor<'de> for Visitor<T> {
      type Value = Id<T>;

      fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a whim snowflake type")
      }

      fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
      where
        E: DeError,
      {
        let value = u64::try_from(v)
          .map_err(|_| DeError::invalid_value(Unexpected::Signed(v), &"nonzero u64"))?;

        self.visit_u64(value)
      }

      fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        let value = NonZeroU64::new(v)
          .ok_or_else(|| DeError::invalid_value(Unexpected::Unsigned(v), &"nonzero u64"))?;

        Ok(Id::<T>::from_nonzero(value))
      }

      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
      where
        E: DeError,
      {
        let value = v.parse().map_err(|_| {
          let unexpected = Unexpected::Str(v);
          DeError::invalid_value(unexpected, &"nonzero u64 string")
        })?;

        self.visit_u64(value)
      }
    }

    deserializer.deserialize_any(Visitor(PhantomData))
  }
}

impl<T: Marker> serde::Serialize for Id<T> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let value = self.value.get().to_string();
    serializer.collect_str(&value)
  }
}

impl<'q, T: Marker> sqlx::Encode<'q, sqlx::Postgres> for Id<T> {
  // already checked
  #[allow(clippy::cast_sign_loss)]
  fn encode_by_ref(
    &self,
    buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
  ) -> sqlx::encode::IsNull {
    const I64_AS_U64_MAX: u64 = i64::MAX as u64;
    if self.value.get() > I64_AS_U64_MAX {
      sqlx::encode::IsNull::Yes
    } else {
      <i64 as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&(self.value.get() as i64), buf)
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::types::id::marker::AnyMarker;
  use serde_test::Token;
  use static_assertions::{assert_eq_size, assert_impl_all};

  assert_eq_size!(Id<AnyMarker>, u64);
  assert_impl_all!(Id<AnyMarker>:
    Debug, Display, Clone, Copy, Send, Sync, Hash, sqlx::Decode<'static, sqlx::Postgres>,
    sqlx::Encode<'static, sqlx::Postgres>, sqlx::Type<sqlx::Postgres>
  );

  #[test]
  #[should_panic]
  fn test_new_with_zero() {
    _ = Id::<AnyMarker>::new(0);
  }

  #[test]
  fn test_initializers() {
    assert!(Id::<AnyMarker>::new_checked(0).is_none());
    assert_eq!(Some(1), Id::<AnyMarker>::new_checked(1).map(Id::get));
  }

  #[test]
  fn test_fmt_display_impl() {
    assert_eq!("1234567890", Id::<AnyMarker>::new(1234567890).to_string());
  }

  #[test]
  fn test_fmt_debug_impl() {
    use heck::ToSnakeCase;

    // for `marker` module
    assert_eq!(
      "Id::<AnyMarker>(1234567890)",
      format!("{:?}", Id::<AnyMarker>::new(1234567890))
    );

    #[derive(Debug, PartialEq, Eq)]
    struct DummyMarker;
    impl marker::Marker for DummyMarker {}
    impl crate::internal::Sealed for DummyMarker {}

    // This is just in case if people will fork and rename
    // with my project under the hood. :)
    let expected = format!(
      "Id::<{}::types::id::tests::test_fmt_debug_impl::DummyMarker>(1234567890)",
      env!("CARGO_PKG_NAME").to_snake_case()
    );
    assert_eq!(
      expected,
      format!("{:?}", Id::<DummyMarker>::new(1234567890))
    );
  }

  #[test]
  fn test_serde_impl() {
    let id = Id::<AnyMarker>::new(1234567890);
    serde_test::assert_de_tokens(&id, &[Token::U64(1234567890)]);
    serde_test::assert_de_tokens(&id, &[Token::Str("1234567890")]);
    serde_test::assert_de_tokens(&id, &[Token::I64(1234567890)]);
    serde_test::assert_ser_tokens(&id, &[Token::Str("1234567890")]);
  }
}
