use serde::{de::IgnoredAny, ser::SerializeMap, Deserialize, Serialize};
use std::{fmt::Display, marker::PhantomData};

use super::{codes, ErrorCategory, ErrorType};

struct TypedDataDeserializer<T: ErrorCategory>(Option<u32>, PhantomData<T>);

impl<'de, T: ErrorCategory> serde::de::DeserializeSeed<'de>
  for TypedDataDeserializer<T>
{
  type Value = ErrorType<T>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    T::deserialize_data(self.0, deserializer).map(ErrorType::Specific)
  }
}

#[derive(Debug, Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum Field {
  Code,
  Subcode,
  Message,
  Data,
}

struct Visitor<T: ErrorCategory>(PhantomData<T>);

impl<'de, T: ErrorCategory> serde::de::Visitor<'de> for Visitor<T> {
  type Value = ErrorType<T>;

  fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Capwat error type")
  }

  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::MapAccess<'de>,
  {
    let mut code: Option<u32> = None;
    let mut subcode: Option<u32> = None;
    let mut message: Option<String> = None;
    let mut data: Option<serde_value::Value> = None;

    while let Some(field) = map.next_key::<Field>()? {
      match field {
        Field::Code => {
          if code.is_some() {
            return Err(serde::de::Error::missing_field("code"));
          }
          code = Some(map.next_value()?);
        },
        Field::Subcode => {
          if subcode.is_some() {
            return Err(serde::de::Error::missing_field("subcode"));
          }
          subcode = Some(map.next_value()?);
        },
        Field::Message => {
          if message.is_some() {
            return Err(serde::de::Error::missing_field("message"));
          }
          message = Some(map.next_value()?);
        },
        Field::Data => {
          // If the code is equal to the generic, we can proceed to most
          // efficient way to parse data which is the `TypedDataDeserializer`.
          if code.map(|v| T::code() == v).unwrap_or_default() {
            // But first, we need to check if `message` is not missing
            message
              .ok_or_else(|| serde::de::Error::missing_field("message"))?;

            // must be in order otherwise, boom an error
            let data = map.next_value_seed(TypedDataDeserializer::<T>(
              subcode,
              PhantomData,
            ))?;

            // Skip any remaining fields and values
            while map.next_entry::<Field, IgnoredAny>()?.is_some() {
              match field {
                Field::Code => {
                  return Err(serde::de::Error::missing_field("code"))
                },
                Field::Subcode => {
                  if subcode.is_some() {
                    return Err(serde::de::Error::missing_field("subcode"));
                  }
                },
                Field::Message => {
                  return Err(serde::de::Error::missing_field("message"))
                },
                Field::Data => {
                  return Err(serde::de::Error::missing_field("data"));
                },
              }
            }
            return Ok(data);
          }
          if data.is_some() {
            return Err(serde::de::Error::missing_field("data"));
          }
          data = Some(map.next_value()?);
        },
      }
    }

    let code = code.ok_or_else(|| serde::de::Error::missing_field("code"))?;
    let message =
      message.ok_or_else(|| serde::de::Error::missing_field("message"))?;

    match code {
      codes::INTERNAL => Ok(ErrorType::Internal),
      codes::READONLY_MODE => Ok(ErrorType::Readonly),
      codes::UNAUTHORIZED => Ok(ErrorType::Unauthorized),

      _ if code == T::code() => {
        // We move on to the least efficient method of parsing
        let deserializer = serde_value::ValueDeserializer::new(
          data.unwrap_or_else(|| serde_value::Value::Option(None)),
        );

        T::deserialize_data(subcode, deserializer).map(ErrorType::Specific)
      },
      _ => {
        Ok(ErrorType::Unknown(super::Unknown { code, subcode, message, data }))
      },
    }
  }
}

impl<'de, T: ErrorCategory> Deserialize<'de> for ErrorType<T> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_map(Visitor(PhantomData))
  }
}

struct MessageTypePrinter<'a, T: ErrorCategory>(&'a ErrorType<T>);

impl<'a, T: ErrorCategory> Display for MessageTypePrinter<'a, T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.0.message(f)
  }
}

impl<'a, T: ErrorCategory> Serialize for MessageTypePrinter<'a, T> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.collect_str(self)
  }
}

struct DataSerializer<'a, T: ErrorCategory>(&'a ErrorType<T>);

impl<'a, T: ErrorCategory> Serialize for DataSerializer<'a, T> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let this = self.0;
    assert!(this.needs_data_serialization());
    this.serialize_data(serializer)
  }
}

impl<T: ErrorCategory> Serialize for ErrorType<T> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut len = 2;
    let subcode = match self.subcode() {
      Some(n) => {
        len += 1;
        Some(n)
      },
      None => None,
    };
    let needs_data_ser = self.needs_data_serialization();
    if needs_data_ser {
      len += 1;
    }

    let mut map = serializer.serialize_map(Some(len))?;
    map.serialize_entry("code", &self.code())?;
    if let Some(subcode) = subcode {
      map.serialize_entry("subcode", &subcode)?;
    }
    map.serialize_entry("message", &MessageTypePrinter(self))?;

    if needs_data_ser {
      map.serialize_entry("data", &DataSerializer(self))?;
    }

    map.end()
  }
}

#[cfg(test)]
mod tests {
  use serde::{de::IgnoredAny, Deserialize};
  use serde_test::Token;
  use thiserror::Error;

  use crate::error::{ErrorCategory, ErrorType};

  #[derive(Debug, Error, PartialEq, Eq)]
  #[error("Oops!")]
  enum OopsieError {
    General,
    WithSubcode,
    WithData(&'static str),
  }

  impl ErrorCategory for OopsieError {
    fn code() -> u32 {
      1
    }

    fn subcode(&self) -> Option<u32> {
      match self {
        Self::WithSubcode => Some(1),
        Self::WithData(..) | Self::General => None,
      }
    }

    #[cfg(feature = "server_impl")]
    fn server_message(
      &self,
      _f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
      Ok(())
    }

    fn message(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.write_str("Oops")
    }

    fn needs_data_serialization(&self) -> bool {
      true
    }

    fn serialize_data<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: serde::Serializer,
    {
      match self {
        Self::General | Self::WithSubcode => serializer.serialize_str("Hi"),
        Self::WithData(data) => serializer.serialize_str(data),
      }
    }

    fn deserialize_data<'de, D>(
      subcode: Option<u32>,
      deserializer: D,
    ) -> Result<Self, D::Error>
    where
      D: serde::Deserializer<'de>,
      Self: Sized,
    {
      IgnoredAny::deserialize(deserializer)?;
      match subcode {
        Some(1) => Ok(Self::WithSubcode),
        None => Ok(Self::General),
        _ => Err(serde::de::Error::custom("invalid subcode of OopsieError")),
      }
    }
  }

  #[test]
  fn test_deserialize() {
    #[allow(clippy::needless_pass_by_value)]
    fn test_simple_variant(error: ErrorType<OopsieError>) {
      let serialized = serde_json::to_string(&error).unwrap();
      let new_data =
        serde_json::from_str::<ErrorType<OopsieError>>(&serialized).unwrap();

      assert_eq!(error, new_data);
    }

    test_simple_variant(ErrorType::Internal);
    test_simple_variant(ErrorType::Readonly);
    test_simple_variant(ErrorType::Unauthorized);

    serde_test::assert_tokens(
      &ErrorType::Specific(OopsieError::General),
      &[
        Token::Map { len: Some(3) },
        Token::Str("code"),
        Token::U32(1),
        Token::Str("message"),
        Token::Str("Oops"),
        Token::Str("data"),
        Token::Str("Hi"),
        Token::MapEnd,
      ],
    );

    serde_test::assert_tokens(
      &ErrorType::Specific(OopsieError::WithSubcode),
      &[
        Token::Map { len: Some(4) },
        Token::Str("code"),
        Token::U32(1),
        Token::Str("subcode"),
        Token::U32(1),
        Token::Str("message"),
        Token::Str("Oops"),
        Token::Str("data"),
        Token::Str("Hi"),
        Token::MapEnd,
      ],
    );
  }

  #[test]
  fn test_serialize() {
    let other =
      serde_json::to_string(&ErrorType::<OopsieError>::Internal).unwrap();
    assert_eq!(
      r#"{"code":1,"message":"Internal server occurred. Please try again later."}"#,
      other
    );

    let other =
      serde_json::to_string(&ErrorType::<OopsieError>::Readonly).unwrap();
    assert_eq!(
      r#"{"code":2,"message":"This service is currently in read only mode. Please try again later."}"#,
      other
    );

    let other =
      serde_json::to_string(&ErrorType::<OopsieError>::Unauthorized).unwrap();

    assert_eq!(
      r#"{"code":3,"message":"You do not have permission to access this information"}"#,
      other
    );

    /////////////////////////////////////////////////////////////////////////////
    let serialized =
      serde_json::to_string(&ErrorType::Specific(OopsieError::General))
        .unwrap();

    assert_eq!(r#"{"code":1,"message":"Oops","data":"Hi"}"#, serialized);

    let serialized =
      serde_json::to_string(&ErrorType::Specific(OopsieError::WithSubcode))
        .unwrap();

    assert_eq!(
      r#"{"code":1,"subcode":1,"message":"Oops","data":"Hi"}"#,
      serialized
    );

    let serialized = serde_json::to_string(&ErrorType::Specific(
      OopsieError::WithData("So cool!"),
    ))
    .unwrap();

    assert_eq!(r#"{"code":1,"message":"Oops","data":"So cool!"}"#, serialized);
  }
}
