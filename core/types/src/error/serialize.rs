use serde::{de::IgnoredAny, ser::SerializeMap, Deserialize, Serialize};
use serde_value::Value;

use super::Error;
use crate::error::{consts, ErrorCode, LoginUser, Unknown};

impl<'de> Deserialize<'de> for Error {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_map(ErrorVisitor)
  }
}

struct ErrorVisitor;

impl<'de> serde::de::Visitor<'de> for ErrorVisitor {
  type Value = Error;

  fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Capwat error type")
  }

  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where
    A: serde::de::MapAccess<'de>,
  {
    // TODO: Optimize this code below especially to the `data` field.
    //
    // This is not very efficient and unnecessary to do if `data` field
    // is present but not required and it wastes too much time deserializing.
    // We need to rush sometimes.
    #[derive(Debug, Deserialize)]
    #[serde(field_identifier, rename_all = "snake_case")]
    enum Field {
      Code,
      Subcode,
      Message,
      Data,
    }

    let mut code: Option<ErrorCode> = None;
    let mut subcode: Option<u32> = None;
    let mut message: Option<String> = None;
    let mut data: Option<Value> = None;

    loop {
      let key = match map.next_key() {
        Ok(Some(key)) => key,
        Ok(None) => break,
        Err(_) => {
          map.next_value::<IgnoredAny>()?;
          continue;
        },
      };
      match key {
        Field::Code => {
          if code.is_some() {
            return Err(serde::de::Error::duplicate_field("code"));
          }
          code = Some(map.next_value()?);
        },
        Field::Subcode => {
          if subcode.is_some() {
            return Err(serde::de::Error::duplicate_field("subcode"));
          }
          subcode = Some(map.next_value()?);
        },
        Field::Message => {
          if message.is_some() {
            return Err(serde::de::Error::duplicate_field("message"));
          }
          message = Some(map.next_value()?);
        },
        Field::Data => {
          if data.is_some() {
            return Err(serde::de::Error::duplicate_field("data"));
          }
          data = Some(map.next_value()?);
        },
      }
    }

    let code = code.ok_or_else(|| serde::de::Error::missing_field("code"))?;
    let message =
      message.ok_or_else(|| serde::de::Error::missing_field("message"))?;

    Ok(match code {
      ErrorCode::Internal => Error::Internal,
      ErrorCode::ReadonlyMode => Error::ReadonlyMode,
      ErrorCode::NotAuthenticated => Error::NotAuthenticated,
      ErrorCode::InvalidFormBody => Error::InvalidFormBody({
        data
          .ok_or_else(|| serde::de::Error::missing_field("message"))?
          .deserialize_into()
          .map_err(serde::de::Error::custom)?
      }),
      ErrorCode::LoginUser => Error::LoginUser({
        let subcode =
          subcode.ok_or_else(|| serde::de::Error::missing_field("subcode"))?;

        match subcode {
          consts::login_user::INVALID_CREDIENTIALS_CODE => {
            Box::new(LoginUser::InvalidCredientials)
          },
          consts::login_user::BANNED_CODE => {
            let data: crate::error::variants::LoginUserBanData = data
              .ok_or_else(|| serde::de::Error::missing_field("message"))?
              .deserialize_into()
              .map_err(serde::de::Error::custom)?;

            Box::new(LoginUser::Banned(data))
          },
          _ => {
            return Err(serde::de::Error::custom(format_args!(
              "unknown subcode of LoginUser ({subcode})"
            )))
          },
        }
      }),
      code @ ErrorCode::Unknown(..) => {
        Error::Unknown(Box::new(Unknown { code, subcode, message, data }))
      },
    })
  }
}

#[allow(clippy::single_match_else)]
impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      Error::InvalidFormBody(data) => {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("code", &self.code())?;
        map.serialize_entry("message", &self.message())?;
        map.serialize_entry("data", &data)?;
        map.end()
      },
      Error::LoginUser(data) => match data.as_ref() {
        LoginUser::InvalidCredientials => {
          let mut map = serializer.serialize_map(Some(3))?;
          map.serialize_entry("code", &self.code())?;
          map.serialize_entry(
            "subcode",
            &consts::login_user::INVALID_CREDIENTIALS_CODE,
          )?;
          map.serialize_entry("message", &self.message())?;
          map.end()
        },
        LoginUser::Banned(data) => {
          let mut map = serializer.serialize_map(Some(4))?;
          map.serialize_entry("code", &self.code())?;
          map.serialize_entry(
            "subcode",
            &consts::login_user::INVALID_CREDIENTIALS_CODE,
          )?;
          map.serialize_entry("message", &self.message())?;
          map.serialize_entry("data", data)?;
          map.end()
        },
      },
      Error::Unknown(meta) => {
        let mut len = 2;
        if meta.subcode.is_some() {
          len += 1;
        }
        if meta.data.is_some() {
          len += 1;
        }

        let mut map = serializer.serialize_map(Some(len))?;
        map.serialize_entry("code", &meta.code)?;
        if let Some(subcode) = meta.subcode {
          map.serialize_entry("subcode", &subcode)?;
        }
        map.serialize_entry("message", &meta.message)?;
        if let Some(data) = meta.data.as_ref() {
          map.serialize_entry("data", &data)?;
        }
        map.end()
      },
      _ => {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("code", &self.code())?;
        map.serialize_entry("message", &self.message())?;
        map.end()
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::error::Error;
  use crate::error::{consts, ErrorCode};

  use serde_test::{assert_tokens, Token};
  use serde_value::Value;
  use std::collections::BTreeMap;

  #[test]
  fn test_unknown_errors() {
    let mut map = BTreeMap::new();
    map.insert(Value::String("Hello".into()), Value::String("World!".into()));

    let value = Error::Unknown(Box::new(super::Unknown {
      code: ErrorCode::Unknown(u32::MAX),
      subcode: Some(u32::MAX),
      message: "Hello!".into(),
      data: Some(Value::Map(map.clone())),
    }));
    assert_tokens(
      &value,
      &[
        Token::Map { len: Some(4) },
        Token::Str("code"),
        Token::U32(u32::MAX),
        Token::Str("subcode"),
        Token::U32(u32::MAX),
        Token::Str("message"),
        Token::String("Hello!"),
        Token::Str("data"),
        Token::Map { len: Some(1) },
        Token::Str("Hello"),
        Token::Str("World!"),
        Token::MapEnd,
        Token::MapEnd,
      ],
    );

    let value = Error::Unknown(Box::new(super::Unknown {
      code: ErrorCode::Unknown(u32::MAX),
      subcode: None,
      message: "Hello!".into(),
      data: Some(Value::Map(map)),
    }));
    assert_tokens(
      &value,
      &[
        Token::Map { len: Some(3) },
        Token::Str("code"),
        Token::U32(u32::MAX),
        Token::Str("message"),
        Token::String("Hello!"),
        Token::Str("data"),
        Token::Map { len: Some(1) },
        Token::Str("Hello"),
        Token::Str("World!"),
        Token::MapEnd,
        Token::MapEnd,
      ],
    );

    let value = Error::Unknown(Box::new(super::Unknown {
      code: ErrorCode::Unknown(u32::MAX),
      subcode: Some(u32::MAX),
      message: "Hello!".into(),
      data: None,
    }));
    assert_tokens(
      &value,
      &[
        Token::Map { len: Some(3) },
        Token::Str("code"),
        Token::U32(u32::MAX),
        Token::Str("subcode"),
        Token::U32(u32::MAX),
        Token::Str("message"),
        Token::String("Hello!"),
        Token::MapEnd,
      ],
    );

    let value = Error::Unknown(Box::new(super::Unknown {
      code: ErrorCode::Unknown(u32::MAX),
      subcode: None,
      message: "Hello!".into(),
      data: None,
    }));
    assert_tokens(
      &value,
      &[
        Token::Map { len: Some(2) },
        Token::Str("code"),
        Token::U32(u32::MAX),
        Token::Str("message"),
        Token::String("Hello!"),
        Token::MapEnd,
      ],
    );
  }

  #[test]
  fn test_simple_variants() {
    static VARIANTS: &[(Error, ErrorCode, &str)] = &[
      (Error::Internal, ErrorCode::Internal, consts::INTERNAL_MSG),
      (
        Error::NotAuthenticated,
        ErrorCode::NotAuthenticated,
        consts::NOT_AUTHENTICATED_MSG,
      ),
      (Error::ReadonlyMode, ErrorCode::ReadonlyMode, consts::READONLY_MODE_MSG),
    ];

    for (info, code, msg) in VARIANTS {
      assert_tokens(
        info,
        &[
          Token::Map { len: Some(2) },
          Token::Str("code"),
          Token::U32(code.as_u32()),
          Token::Str("message"),
          Token::Str(msg),
          Token::MapEnd,
        ],
      );
    }
  }
}
