use serde::{Deserialize, Serialize};
use std::{
  borrow::Cow,
  fmt::{Debug, Display},
};

/// Keeps the raw sensitive data in memory but it cannot be
/// accidentally leaked through the console or logs.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Sensitive<T>(T);

impl<T> Sensitive<T> {
  pub const fn new(value: T) -> Self {
    Self(value)
  }

  pub fn into_inner(self) -> T {
    self.0
  }
}

impl Sensitive<String> {
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl<T> Debug for Sensitive<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("<hidden>").finish()
  }
}

impl<T> Display for Sensitive<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("<hidden>").finish()
  }
}

impl<T> AsRef<T> for Sensitive<T> {
  fn as_ref(&self) -> &T {
    &self.0
  }
}

impl AsRef<str> for Sensitive<String> {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl AsRef<[u8]> for Sensitive<String> {
  fn as_ref(&self) -> &[u8] {
    self.0.as_ref()
  }
}

impl AsRef<[u8]> for Sensitive<Vec<u8>> {
  fn as_ref(&self) -> &[u8] {
    self.0.as_ref()
  }
}

impl<T> AsMut<T> for Sensitive<T> {
  fn as_mut(&mut self) -> &mut T {
    &mut self.0
  }
}

impl AsMut<str> for Sensitive<String> {
  fn as_mut(&mut self) -> &mut str {
    &mut self.0
  }
}

impl std::ops::Deref for Sensitive<String> {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl std::ops::DerefMut for Sensitive<String> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T> From<T> for Sensitive<T> {
  fn from(t: T) -> Self {
    Sensitive(t)
  }
}

impl From<&str> for Sensitive<String> {
  fn from(s: &str) -> Self {
    Sensitive(s.into())
  }
}

impl<T> std::borrow::Borrow<T> for Sensitive<T> {
  fn borrow(&self) -> &T {
    &self.0
  }
}

impl std::borrow::Borrow<str> for Sensitive<String> {
  fn borrow(&self) -> &str {
    &self.0
  }
}

impl<'a> Into<Cow<'a, str>> for Sensitive<String> {
  fn into(self) -> Cow<'a, str> {
    Cow::Owned(self.0)
  }
}

impl<'a> Into<Cow<'a, str>> for &'a Sensitive<String> {
  fn into(self) -> Cow<'a, str> {
    Cow::Borrowed(&self.0)
  }
}

impl<'a> validator::HasLength for Sensitive<String> {
  fn length(&self) -> usize {
    self.0.len()
  }
}

impl<'a> validator::HasLength for &'a Sensitive<String> {
  fn length(&self) -> usize {
    self.0.len()
  }
}

#[cfg(test)]
mod tests {
  use super::Sensitive;

  #[test]
  fn fmt() {
    let value = Sensitive::new("hello");
    assert_eq!(value.to_string(), "<hidden>");
    assert_eq!(format!("{:?}", value), "<hidden>");
  }

  #[test]
  fn has_len() {
    use validator::HasLength;

    let testing = Sensitive::new("Hello".to_string());
    assert_eq!(testing.length(), 5);
  }
}
