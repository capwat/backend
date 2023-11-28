use serde::{ser::SerializeStruct, Deserialize, Serialize};
use std::fmt::Display;

use super::ErrorType;
use crate::error::{ErrorCode, RawError};

#[derive(Debug, Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum Field {
  Code,
  Subcode,
  Message,
  Data,
}

struct Visitor;

// TODO: Replicate twilight's way on deserializing Discord gateway
impl<'de> serde::de::Visitor<'de> for Visitor {
  type Value = ErrorType;

  fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Capwat error type")
  }

  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::MapAccess<'de>,
  {
    let mut code: Option<ErrorCode> = None;
    let mut subcode: Option<u32> = None;
    let mut message: Option<String> = None;
    let mut data: Option<_> = None;

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
      ErrorCode::Internal => Ok(ErrorType::Internal),
      ErrorCode::ReadonlyMode => Ok(ErrorType::ReadonlyMode),
      ErrorCode::NotAuthenticated => Ok(ErrorType::NotAuthenticated),
      ErrorCode::Unknown(..) => {
        Ok(ErrorType::Unknown(RawError { code, subcode, message, data }))
      },
    }
  }
}

impl<'de> Deserialize<'de> for ErrorType {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_map(Visitor)
  }
}

impl Serialize for ErrorType {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    struct ErrorMessage<'a>(&'a ErrorType);

    impl<'a> Display for ErrorMessage<'a> {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.message(f)
      }
    }

    impl<'a> Serialize for ErrorMessage<'a> {
      fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where
        S: serde::Serializer,
      {
        serializer.collect_str(self)
      }
    }

    match self {
      ErrorType::Internal
      | ErrorType::ReadonlyMode
      | ErrorType::NotAuthenticated => {
        let mut map = serializer.serialize_struct("Error", 2)?;
        map.serialize_field("code", &self.code())?;
        map.serialize_field("message", &ErrorMessage(self))?;
        map.end()
      },
      ErrorType::Unknown(data) => data.serialize(serializer),
    }
  }
}
