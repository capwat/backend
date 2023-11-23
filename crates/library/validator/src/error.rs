use indexmap::IndexMap;
use std::borrow::Cow;

fn serialize_index_map<K: Serialize, V: Serialize, S: serde::Serializer>(
  map: &IndexMap<K, V>,
  serializer: S,
) -> Result<S::Ok, S::Error> {
  let mut map_ser = serializer.serialize_map(Some(map.len()))?;
  for (key, value) in map {
    map_ser.serialize_entry(key, value)?;
  }
  map_ser.end()
}

pub struct MessageBuilder(Vec<Cow<'static, str>>);

impl MessageBuilder {
  #[must_use]
  pub const fn new() -> Self {
    Self(Vec::new())
  }

  pub fn insert(&mut self, message: impl Into<Cow<'static, str>>) {
    self.0.push(message.into());
  }

  #[must_use]
  pub fn build(self) -> ValidateError {
    ValidateError::Messages(self.0)
  }
}

pub struct SliceBuilder(Vec<Option<ValidateError>>);

impl SliceBuilder {
  #[must_use]
  pub const fn new() -> Self {
    Self(Vec::new())
  }

  pub fn insert_empty(&mut self) {
    self.0.push(None);
  }

  pub fn insert(&mut self, value: ValidateError) {
    self.0.push(if value.is_empty() { None } else { Some(value) });
  }

  #[must_use]
  pub fn build(self) -> ValidateError {
    ValidateError::Slice(self.0)
  }
}

pub struct FieldBuilder(IndexMap<Cow<'static, str>, ValidateError>);

#[allow(clippy::new_without_default)]
impl FieldBuilder {
  #[must_use]
  pub fn new() -> Self {
    Self(IndexMap::default())
  }

  pub fn insert(&mut self, key: impl Into<Cow<'static, str>>, value: ValidateError) {
    if !value.is_empty() {
      self.0.insert(key.into(), value);
    }
  }

  #[must_use]
  pub fn build(self) -> ValidateError {
    ValidateError::Fields(self.0)
  }
}

// ---------------------------------------------------- //

#[derive(PartialEq, Eq)]
pub enum ValidateError {
  Fields(IndexMap<Cow<'static, str>, ValidateError>),
  Messages(Vec<Cow<'static, str>>),
  Slice(Vec<Option<ValidateError>>),
}

impl std::fmt::Display for ValidateError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Invalid data occurred")
  }
}

impl std::error::Error for ValidateError {}

impl std::fmt::Debug for ValidateError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ValidateError::Fields(n) => n.fmt(f),
      ValidateError::Messages(n) => f.debug_map().entry(&"_errors", &n).finish(),
      ValidateError::Slice(n) => n.fmt(f),
    }
  }
}

impl ValidateError {
  #[must_use]
  pub fn field_builder() -> FieldBuilder {
    FieldBuilder::new()
  }

  #[must_use]
  pub fn msg_builder() -> MessageBuilder {
    MessageBuilder::new()
  }

  #[must_use]
  pub fn slice_builder() -> SliceBuilder {
    SliceBuilder::new()
  }
}

impl ValidateError {
  #[must_use]
  pub fn is_empty(&self) -> bool {
    match self {
      ValidateError::Slice(n) => {
        for e in n {
          if e.is_some() {
            return false;
          }
        }
        true
      },
      ValidateError::Fields(n) => n.is_empty(),
      ValidateError::Messages(n) => n.is_empty(),
    }
  }

  pub fn into_result(self) -> Result<(), Self> {
    if self.is_empty() {
      Ok(())
    } else {
      Err(self)
    }
  }
}

use serde::{ser::SerializeMap, Serialize};

impl<'de> serde::Deserialize<'de> for ValidateError {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
      type Value = ValidateError;

      fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ValidateError type")
      }

      fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
      where
        A: serde::de::MapAccess<'de>,
      {
        let mut fields = IndexMap::new();
        let mut addr_data = None;

        while let Some(key) = map.next_key::<String>()? {
          match key.as_str() {
            "_errors" => {
              if addr_data.is_some() {
                return Err(serde::de::Error::duplicate_field("_errors"));
              }
              addr_data = Some(map.next_value::<Vec<Cow<'static, str>>>()?);
            },
            _ => {
              fields.insert(Cow::Owned(key), map.next_value()?);
            },
          }
        }

        if let Some(data) = addr_data {
          Ok(ValidateError::Messages(data))
        } else if !fields.is_empty() {
          Ok(ValidateError::Fields(fields))
        } else {
          Err(serde::de::Error::custom("error fields must not be empty"))
        }
      }

      fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
      where
        A: serde::de::SeqAccess<'de>,
      {
        let mut list = Vec::new();
        while let Some(element) = seq.next_element()? {
          list.push(element);
        }
        Ok(ValidateError::Slice(list))
      }
    }

    deserializer.deserialize_any(Visitor)
  }
}

impl Serialize for ValidateError {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      ValidateError::Fields(n) => serialize_index_map(n, serializer),
      ValidateError::Messages(n) => {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("_errors", &n)?;
        map.end()
      },
      ValidateError::Slice(n) => n.serialize(serializer),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::Validate;

  use super::*;
  use serde_test::Token;

  #[derive(Debug)]
  struct Hello {
    names: Vec<&'static str>,
    age: u32,
  }

  impl Validate for Hello {
    fn validate(&self) -> Result<(), ValidateError> {
      fn validate_names(names: &[&'static str]) -> Result<(), ValidateError> {
        let mut slice = ValidateError::slice_builder();
        for name in names {
          let mut msg = ValidateError::msg_builder();
          if name.is_empty() {
            msg.insert("Name is empty");
          }
          slice.insert(msg.build());
        }
        slice.build().into_result()
      }

      let mut fields = ValidateError::field_builder();
      if let Err(e) = validate_names(&self.names) {
        fields.insert("name", e);
      }
      {
        let mut msg = ValidateError::msg_builder();
        if self.age == 0 {
          msg.insert("invalid age");
        }
        if self.age > 202 {
          msg.insert("too old");
        }
        fields.insert("age", msg.build());
      }
      fields.build().into_result()
    }
  }

  #[test]
  fn test_debug_fmt() {
    const EXPECTED_FMT_MSG: &str = r#"{"name": [None, Some({"_errors": ["Name is empty"]}), None], "age": {"_errors": ["invalid age"]}}"#;

    let error = Hello { names: vec!["Mike", "", "John"], age: 0 }.validate().unwrap_err();
    assert_eq!(EXPECTED_FMT_MSG, format!("{error:?}"));
  }

  #[test]
  fn test_serde_impl() {
    let error = Hello { names: vec!["Mike", "", "John"], age: 0 }.validate().unwrap_err();
    serde_test::assert_tokens(
      &error,
      &[
        Token::Map { len: Some(2) },
        Token::Str("name"),
        Token::Seq { len: Some(3) },
        Token::None,
        Token::Some,
        Token::Map { len: Some(1) },
        Token::Str("_errors"),
        Token::Seq { len: Some(1) },
        Token::Str("Name is empty"),
        Token::SeqEnd,
        Token::MapEnd,
        Token::None,
        Token::SeqEnd,
        Token::Str("age"),
        Token::Map { len: Some(1) },
        Token::Str("_errors"),
        Token::Seq { len: Some(1) },
        Token::Str("invalid age"),
        Token::SeqEnd,
        Token::MapEnd,
        Token::MapEnd,
      ],
    );
  }

  #[test]
  fn validate_error_is_empty() {
    assert!(MessageBuilder::new().build().is_empty());
    assert!(FieldBuilder::new().build().is_empty());

    let mut msg = MessageBuilder::new();
    msg.insert("Hello world!");
    assert!(!msg.build().is_empty());

    {
      let mut msg = MessageBuilder::new();
      msg.insert("Hello world!");

      let mut err = FieldBuilder::new();
      err.insert("microbar", msg.build());
      assert!(!err.build().is_empty());
    }

    {
      let mut msg = MessageBuilder::new();
      msg.insert("Hello world!");

      let mut err = SliceBuilder::new();
      err.insert(msg.build());
      assert!(!err.build().is_empty());

      let mut err = SliceBuilder::new();
      err.insert_empty();
      assert!(err.build().is_empty());
    }
  }
}
