mod marker;

use crate::Timestamp;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::marker::PhantomData;
use std::num::NonZeroU64;

pub use marker::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id<T: Marker> {
    // required for implementing other server required traits
    pub(crate) value: NonZeroU64,
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
        Self { value: n, phantom: PhantomData }
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
        Id { value: self.value, phantom: PhantomData }
    }

    #[must_use]
    pub fn timestamp(self) -> Timestamp {
        Timestamp::from_snowflake(self.value)
    }
}

impl<T: Marker> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Id")?;

        let type_name = std::any::type_name::<T>();
        if let Some(slice) = type_name.split("::").last() {
            f.write_str("<")?;
            f.write_str(slice)?;
            f.write_str(">")?;
        }

        f.write_str("(")?;
        Debug::fmt(&self.value.get(), f)?;
        f.write_str(")")
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

impl<T: Marker> PartialEq<u64> for Id<T> {
    fn eq(&self, other: &u64) -> bool {
        self.value.get().eq(other)
    }
}

impl<T: Marker> PartialOrd<u64> for Id<T> {
    fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
        self.value.get().partial_cmp(other)
    }
}

impl<T: Marker> From<Id<T>> for u64 {
    fn from(value: Id<T>) -> Self {
        value.get()
    }
}

impl<T: Marker> From<NonZeroU64> for Id<T> {
    fn from(value: NonZeroU64) -> Self {
        Self::from_nonzero(value)
    }
}

impl<T: Marker> From<Id<T>> for NonZeroU64 {
    fn from(value: Id<T>) -> Self {
        value.into_nonzero()
    }
}

impl<'de, T: Marker> serde::Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error as DeError, Unexpected};

        struct Visitor<T: Marker>(PhantomData<T>);

        impl<'de, T: Marker> serde::de::Visitor<'de> for Visitor<T> {
            type Value = Id<T>;

            fn expecting(
                &self,
                f: &mut std::fmt::Formatter<'_>,
            ) -> std::fmt::Result {
                f.write_str("a Capwat snowflake type")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                let value = u64::try_from(v).map_err(|_| {
                    DeError::invalid_value(
                        Unexpected::Signed(v),
                        &"nonzero u64",
                    )
                })?;

                self.visit_u64(value)
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let value = NonZeroU64::new(v).ok_or_else(|| {
                    DeError::invalid_value(
                        Unexpected::Unsigned(v),
                        &"nonzero u64",
                    )
                })?;

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

#[cfg(test)]
mod tests {
    use super::marker::AnyMarker;
    use super::*;

    use serde::{Deserialize, Serialize};
    use serde_test::Token;
    use static_assertions::{assert_eq_size, assert_impl_all};

    assert_eq_size!(Id::<AnyMarker>, u64);

    assert_impl_all!(Id<AnyMarker>: Debug, Display, Clone,
      Copy, Send, Sync, Hash, Deserialize<'static>, Serialize,
      PartialEq, Eq, PartialOrd, Ord, PartialEq<u64>, PartialOrd<u64>,
    );

    #[test]
    #[should_panic(expected = "value is zero")]
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
        assert_eq!(
            "1234567890",
            Id::<AnyMarker>::new(1_234_567_890).to_string()
        );
    }

    #[test]
    fn test_fmt_debug_impl() {
        #[derive(Debug, PartialEq, Eq)]
        struct DummyMarker;

        impl marker::Marker for DummyMarker {}
        impl crate::internal::Sealed for DummyMarker {}

        // for `marker` module
        assert_eq!(
            "Id<AnyMarker>(1234567890)",
            format!("{:?}", Id::<AnyMarker>::new(1_234_567_890))
        );

        // This is just in case if people will fork and rename
        // with my project under the hood. :)
        assert_eq!(
            "Id<DummyMarker>(1234567890)",
            format!("{:?}", Id::<DummyMarker>::new(1_234_567_890))
        );
    }

    #[test]
    fn test_serde_impl() {
        let id = Id::<AnyMarker>::new(1_234_567_890);
        serde_test::assert_de_tokens(&id, &[Token::U64(1_234_567_890)]);
        serde_test::assert_de_tokens(&id, &[Token::Str("1234567890")]);
        serde_test::assert_de_tokens(&id, &[Token::I64(1_234_567_890)]);
        serde_test::assert_ser_tokens(&id, &[Token::Str("1234567890")]);
    }
}
