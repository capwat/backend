use std::fmt::Display;

use crate::util::Sensitive;

/// This type tells whether the value is automatically generated
/// from a generator or manually set (from deserialization or declaration).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaybeGenerated<T> {
  Generated(T),
  Set(T),
}

impl<T: Default> Default for MaybeGenerated<T> {
  /// When created, it will automatically set to `Generated` variant.
  fn default() -> Self {
    MaybeGenerated::Generated(T::default())
  }
}

impl<T: std::fmt::Debug> std::fmt::Debug for MaybeGenerated<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Generated(..) => write!(f, "Generated("),
      Self::Set(..) => write!(f, "Set("),
    }?;
    self.value().fmt(f)?;
    write!(f, ")")
  }
}

impl<T: Display> std::fmt::Display for MaybeGenerated<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.value().fmt(f)
  }
}

impl<T> MaybeGenerated<T> {
  pub const fn new(value: T) -> Self {
    Self::Set(value)
  }

  pub const fn is_generated(&self) -> bool {
    matches!(self, Self::Generated(..))
  }

  pub const fn is_set(&self) -> bool {
    matches!(self, Self::Set(..))
  }

  pub const fn value(&self) -> &T {
    match self {
      MaybeGenerated::Set(n) | MaybeGenerated::Generated(n) => n,
    }
  }

  pub fn value_mut(&mut self) -> &mut T {
    match self {
      MaybeGenerated::Set(n) | MaybeGenerated::Generated(n) => n,
    }
  }
}

impl<T> AsRef<T> for MaybeGenerated<Sensitive<T>> {
  fn as_ref(&self) -> &T {
    self.value().as_ref()
  }
}

impl<T> AsRef<T> for MaybeGenerated<T> {
  fn as_ref(&self) -> &T {
    self.value()
  }
}

impl AsRef<str> for MaybeGenerated<String> {
  fn as_ref(&self) -> &str {
    self.value()
  }
}

impl AsRef<[u8]> for MaybeGenerated<String> {
  fn as_ref(&self) -> &[u8] {
    self.value().as_bytes()
  }
}

impl AsRef<[u8]> for MaybeGenerated<Vec<u8>> {
  fn as_ref(&self) -> &[u8] {
    self.value()
  }
}

impl<T> AsMut<T> for MaybeGenerated<T> {
  fn as_mut(&mut self) -> &mut T {
    self.value_mut()
  }
}

impl AsMut<str> for MaybeGenerated<String> {
  fn as_mut(&mut self) -> &mut str {
    self.value_mut()
  }
}

impl<T> AsMut<T> for MaybeGenerated<Sensitive<T>> {
  fn as_mut(&mut self) -> &mut T {
    self.value_mut().as_mut()
  }
}

impl AsMut<str> for MaybeGenerated<Sensitive<String>> {
  fn as_mut(&mut self) -> &mut str {
    self.value_mut().as_mut()
  }
}

impl std::ops::Deref for MaybeGenerated<Sensitive<String>> {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.value()
  }
}

impl std::ops::DerefMut for MaybeGenerated<Sensitive<String>> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.value_mut()
  }
}

impl std::ops::Deref for MaybeGenerated<String> {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.value()
  }
}

impl std::ops::DerefMut for MaybeGenerated<String> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.value_mut()
  }
}

impl<T> From<T> for MaybeGenerated<T> {
  fn from(t: T) -> Self {
    MaybeGenerated::Set(t)
  }
}

impl From<&str> for MaybeGenerated<String> {
  fn from(s: &str) -> Self {
    MaybeGenerated::Set(s.into())
  }
}

impl<T> std::borrow::Borrow<T> for MaybeGenerated<T> {
  fn borrow(&self) -> &T {
    self.value()
  }
}

impl std::borrow::Borrow<str> for MaybeGenerated<String> {
  fn borrow(&self) -> &str {
    self.value()
  }
}

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for MaybeGenerated<T> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    Ok(Self::Set(T::deserialize(deserializer)?))
  }
}

impl<T: serde::Serialize> serde::Serialize for MaybeGenerated<T> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    self.value().serialize(serializer)
  }
}

// impl<T> rand::distributions::Distribution<MaybeGenerated<T>> for rand::distributions::Standard
// where
//   rand::distributions::Standard: rand::distributions::Distribution<T>,
// {
//   fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> MaybeGenerated<T> {
//     MaybeGenerated::Generated(
//       <rand::distributions::Standard as rand::distributions::Distribution<T>>::sample(self, rng),
//     )
//   }
// }

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;

  use super::MaybeGenerated;
  use serde::{Deserialize, Serialize};
  use serde_test::Token;

  // #[test]
  // fn test_generator_impls() {
  //   let value = MaybeGenerated::<bool>::default();
  //   assert_eq!(MaybeGenerated::Generated(false), value);

  //   let number = rand::random::<MaybeGenerated<u64>>();
  //   assert!(number.is_generated());
  // }

  #[test]
  fn test_serialization() {
    #[derive(Deserialize, Serialize)]
    struct TestStruct {
      #[serde(default = "generator")]
      result: MaybeGenerated<String>,
    }

    fn generator() -> MaybeGenerated<String> {
      MaybeGenerated::Generated("auto-generated".to_string())
    }

    serde_test::assert_tokens(
      &MaybeGenerated::Set("Hello".to_string()),
      &[Token::Str("Hello")],
    );

    let mut map = BTreeMap::new();
    map.insert(
      serde_value::Value::String("result".into()),
      serde_value::Value::String("set".into()),
    );

    let output = serde_value::Value::Map(map)
      .deserialize_into::<TestStruct>()
      .unwrap();

    assert_eq!(MaybeGenerated::Set("set".to_string()), output.result);

    let output = serde_value::Value::Map(Default::default())
      .deserialize_into::<TestStruct>()
      .unwrap();

    assert_eq!(generator(), output.result);
  }
}
