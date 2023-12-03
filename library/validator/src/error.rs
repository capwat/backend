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

pub struct MessageBuilder(Option<Vec<Cow<'static, str>>>);

// We're explicitly know that the inner type of it contains a value
#[allow(clippy::unwrap_used)]
impl MessageBuilder {
  #[must_use]
  pub const fn new() -> Self {
    Self(Some(Vec::new()))
  }

  pub fn insert(&mut self, message: impl Into<Cow<'static, str>>) -> &mut Self {
    self.0.as_mut().unwrap().push(message.into());
    self
  }

  #[must_use]
  pub fn build(&mut self) -> ValidateError {
    ValidateError::Messages(self.0.take().unwrap())
  }
}

pub struct SliceBuilder(Option<Vec<Option<ValidateError>>>);

// We're explicitly know that the inner type of it contains a value
#[allow(clippy::unwrap_used)]
impl SliceBuilder {
  #[must_use]
  pub const fn new() -> Self {
    Self(Some(Vec::new()))
  }

  pub fn insert_empty(&mut self) -> &mut Self {
    self.0.as_mut().unwrap().push(None);
    self
  }

  pub fn insert(&mut self, value: ValidateError) -> &mut Self {
    self.0.as_mut().unwrap().push(if value.is_empty() {
      None
    } else {
      Some(value)
    });
    self
  }

  #[must_use]
  pub fn build(&mut self) -> ValidateError {
    ValidateError::Slice(self.0.take().unwrap())
  }
}

pub struct FieldBuilder(Option<IndexMap<Cow<'static, str>, ValidateError>>);

// We're explicitly know that the inner type of it contains a value
#[allow(clippy::new_without_default)]
#[allow(clippy::unwrap_used)]
impl FieldBuilder {
  #[must_use]
  pub fn new() -> Self {
    Self(Some(IndexMap::default()))
  }

  pub fn insert(
    &mut self,
    key: impl Into<Cow<'static, str>>,
    value: ValidateError,
  ) -> &mut Self {
    if !value.is_empty() {
      self.0.as_mut().unwrap().insert(key.into(), value);
    }
    self
  }

  #[must_use]
  pub fn build(&mut self) -> ValidateError {
    ValidateError::Fields(self.0.take().unwrap())
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
      ValidateError::Messages(n) => {
        f.debug_map().entry(&"_errors", &n).finish()
      },
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

  #[must_use]
  pub fn message(message: impl Into<Cow<'static, str>>) -> Self {
    MessageBuilder::new().insert(message.into()).build()
  }
}

impl ValidateError {
  #[must_use]
  pub fn is_empty(&self) -> bool {
    match self {
      ValidateError::Slice(n) => n.iter().all(std::option::Option::is_none),
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

use serde::{de::IgnoredAny, ser::SerializeMap, Serialize};

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
          if key.as_str() == "_errors" {
            if addr_data.is_some() {
              return Err(serde::de::Error::duplicate_field("_errors"));
            }
            addr_data = Some(map.next_value::<Vec<Cow<'static, str>>>()?);
            continue;
          }

          if addr_data.is_none() {
            fields.insert(Cow::Owned(key), map.next_value()?);
          } else {
            map.next_value::<IgnoredAny>()?;
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

  fn validate_age(age: u32) -> Result<(), ValidateError> {
    if age == 0 {
      return Err(ValidateError::msg_builder().insert("Invalid age").build());
    }
    if age > 202 {
      return Err(ValidateError::msg_builder().insert("Too old").build());
    }
    Ok(())
  }

  impl Validate for Hello {
    fn validate(&self) -> Result<(), ValidateError> {
      let mut fields = ValidateError::field_builder();
      if let Err(e) = validate_names(&self.names) {
        fields.insert("name", e);
      }

      if let Err(e) = validate_age(self.age) {
        fields.insert("age", e);
      }

      fields.build().into_result()
    }
  }

  #[test]
  fn test_debug_fmt() {
    const EXPECTED_FMT_MSG: &str = r#"{"name": [None, Some({"_errors": ["Name is empty"]}), None], "age": {"_errors": ["Invalid age"]}}"#;

    let error =
      Hello { names: vec!["Mike", "", "John"], age: 0 }.validate().unwrap_err();

    assert_eq!(EXPECTED_FMT_MSG, format!("{error:?}"));
  }

  #[test]
  fn test_serde_impl() {
    let error =
      Hello { names: vec!["Mike", "", "John"], age: 0 }.validate().unwrap_err();

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
        Token::Str("Invalid age"),
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
