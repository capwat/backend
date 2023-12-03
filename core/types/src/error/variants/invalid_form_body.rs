use indexmap::IndexMap;
use serde::{
  de::{DeserializeSeed, IgnoredAny},
  Deserialize, Serialize,
};
use std::borrow::Cow;

// TODO: Implement non-nested tables
#[derive(Debug, PartialEq, Eq)]
pub enum InvalidFormBody {
  Fields(IndexMap<Cow<'static, str>, InvalidFormBody>),
  Messages(Vec<Cow<'static, str>>),
  Slice(Vec<Option<InvalidFormBody>>),
}

const MAX_ITEMS_PER_NEST: usize = 64;
const NESTED_FIELDS_MAX: usize = 30;

struct Visitor {
  stack: usize,
}

impl<'de> serde::de::Visitor<'de> for Visitor {
  type Value = Option<InvalidFormBody>;

  fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("invalid form body type")
  }

  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::MapAccess<'de>,
  {
    let mut fields = IndexMap::new();
    let mut messages = None;

    while let Some(key) = map.next_key::<String>()? {
      if fields.len() > MAX_ITEMS_PER_NEST {
        return Err(serde::de::Error::custom(format_args!(
          "invalid form fields reached its limit ({MAX_ITEMS_PER_NEST})"
        )));
      }

      let is_errors_key = key.as_str() == "_errors";
      if is_errors_key && messages.is_some() {
        return Err(serde::de::Error::duplicate_field("_errors"));
      }

      if is_errors_key {
        fields.clear();
        messages = Some(map.next_value::<Vec<Cow<'static, str>>>()?);
        continue;
      }

      if messages.is_none() {
        fields.insert(
          Cow::Owned(key),
          map.next_value_seed(Deserializer { stack: self.stack })?,
        );
      } else {
        map.next_value::<IgnoredAny>()?;
      }
    }

    Ok(Some(if let Some(data) = messages {
      InvalidFormBody::Messages(data)
    } else if !fields.is_empty() {
      InvalidFormBody::Fields(fields)
    } else {
      return Err(serde::de::Error::custom("error fields must not be empty"));
    }))
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::SeqAccess<'de>,
  {
    let mut elements = Vec::new();
    while let Some(value) =
      seq.next_element_seed(OptionalDeserializer { stack: self.stack })?
    {
      if elements.len() > MAX_ITEMS_PER_NEST {
        return Err(serde::de::Error::custom(format_args!(
          "invalid form elements reached its limit ({MAX_ITEMS_PER_NEST})"
        )));
      }
      elements.push(value);
    }
    Ok(Some(InvalidFormBody::Slice(elements)))
  }

  fn visit_unit<E>(self) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(None)
  }

  fn visit_none<E>(self) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(None)
  }

  fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    Deserializer { stack: self.stack }.deserialize(deserializer).map(Some)
  }
}

struct OptionalDeserializer {
  stack: usize,
}

impl<'de> serde::de::DeserializeSeed<'de> for OptionalDeserializer {
  type Value = Option<InvalidFormBody>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    if self.stack > NESTED_FIELDS_MAX {
      return Err(serde::de::Error::custom(format_args!(
        "invalid form nested fields reached its limit ({NESTED_FIELDS_MAX})"
      )));
    }
    deserializer.deserialize_any(Visitor { stack: self.stack + 1 })
  }
}

struct Deserializer {
  stack: usize,
}

impl<'de> serde::de::DeserializeSeed<'de> for Deserializer {
  type Value = InvalidFormBody;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    if self.stack > NESTED_FIELDS_MAX {
      return Err(serde::de::Error::custom(format_args!(
        "invalid form nested fields reached its limit ({NESTED_FIELDS_MAX})"
      )));
    }
    deserializer.deserialize_any(Visitor { stack: self.stack + 1 })?.ok_or_else(
      || serde::de::Error::custom("expected error info; got `null` or none"),
    )
  }
}

impl<'de> Deserialize<'de> for InvalidFormBody {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    Deserializer { stack: 0 }.deserialize(deserializer)
  }
}

impl Serialize for InvalidFormBody {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    use serde::ser::SerializeMap;

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

    match self {
      Self::Fields(n) => serialize_index_map(n, serializer),
      Self::Messages(n) => {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("_errors", &n)?;
        map.end()
      },
      Self::Slice(n) => n.serialize(serializer),
    }
  }
}

#[cfg(feature = "server_impl")]
impl From<validator::ValidateError> for InvalidFormBody {
  fn from(value: validator::ValidateError) -> Self {
    use validator::ValidateError;
    match value {
      ValidateError::Fields(data) => Self::Fields(
        data.into_iter().map(|(k, v)| (k, InvalidFormBody::from(v))).collect(),
      ),
      ValidateError::Messages(data) => Self::Messages(data),
      ValidateError::Slice(data) => Self::Slice(
        data.into_iter().map(|v| v.map(InvalidFormBody::from)).collect(),
      ),
    }
  }
}

#[cfg(feature = "server_impl")]
impl From<InvalidFormBody> for validator::ValidateError {
  fn from(value: InvalidFormBody) -> Self {
    use validator::ValidateError;
    match value {
      InvalidFormBody::Fields(data) => Self::Fields(
        data.into_iter().map(|(k, v)| (k, ValidateError::from(v))).collect(),
      ),
      InvalidFormBody::Messages(data) => Self::Messages(data),
      InvalidFormBody::Slice(data) => Self::Slice(
        data.into_iter().map(|v| v.map(ValidateError::from)).collect(),
      ),
    }
  }
}

// Duplicated from validator's error tests
//
// We need to test both validator's error type and InvalidFormBody to
// make sure they recieve the same result when it is serialized.
#[cfg(test)]
#[cfg(feature = "server_impl")]
mod tests {
  use serde_test::Token;
  use validator::{Validate, ValidateError};

  use super::InvalidFormBody;
  use crate::error::consts;

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
      return Err(ValidateError::message("Invalid age"));
    }
    if age > 202 {
      return Err(ValidateError::message("Too old"));
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
  fn test_stack_overflow_protection() {
    let mut input = r#""_errors":["Hi!"]"#.to_string();
    for _ in 0..(super::NESTED_FIELDS_MAX + 10) {
      input = format!("\"error\":{{{input}}}");
    }
    input = format!("{{{input}}}");

    let error = serde_json::from_str::<InvalidFormBody>(&input).unwrap_err();
    assert!(error.is_data());
    assert_eq!(error.line(), 1);
    assert_eq!(error.column(), 279);
    assert!(error.to_string().starts_with(&format!(
      "invalid form nested fields reached its limit ({})",
      super::NESTED_FIELDS_MAX
    )));
  }

  #[test]
  fn test_with_capwat_error() {
    let error =
      crate::error::Error::InvalidFormBody(Box::new(InvalidFormBody::from(
        Hello { names: vec!["Mike", "", "John"], age: 0 }
          .validate()
          .unwrap_err(),
      )));

    serde_test::assert_tokens(
      &error,
      &[
        Token::Map { len: Some(3) },
        Token::Str("code"),
        Token::U32(consts::INVALID_FORM_BODY_CODE),
        Token::Str("message"),
        Token::String(consts::INVALID_FORM_BODY_MSG),
        Token::Str("data"),
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
        Token::MapEnd,
      ],
    );
  }

  #[test]
  fn test_serde_impl() {
    let error = InvalidFormBody::from(
      Hello { names: vec!["Mike", "", "John"], age: 0 }.validate().unwrap_err(),
    );

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
}
