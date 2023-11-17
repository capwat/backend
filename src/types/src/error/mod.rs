use serde::{ser::SerializeMap, Deserialize, Serialize};
use std::{
  borrow::Cow,
  fmt::{Debug, Display},
  marker::PhantomData,
};
use thiserror::Error;

pub mod client;
pub mod codes;
pub mod server;

#[derive(Debug, PartialEq, Eq)]
pub enum RequestType<T> {
  InvalidRequest(client::InvalidRequest),
  Local(T),
  Server(server::Error),
}

impl<T: Primary> RequestType<T> {
  pub const fn invalid_request(value: client::InvalidRequest) -> Self {
    Self::InvalidRequest(value)
  }

  pub const fn local(value: T) -> Self {
    Self::Local(value)
  }

  pub const fn server(value: server::Error) -> Self {
    Self::Server(value)
  }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Request<T> {
  pub error_type: RequestType<T>,
  pub message: String,
}

impl<'de, T: Primary + PrimaryCreator> serde::Deserialize<'de> for Request<T> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let raw = Untyped::deserialize(deserializer)?;
    let result = match &raw.code {
      &codes::INVALID_REQUEST => RequestType::InvalidRequest(
        client::InvalidRequest::from_subcode(raw.subcode, raw.data)
          .map_err(|e| e.into_de_error::<D>())?,
      ),
      &codes::SERVER => RequestType::Server(
        server::Error::from_subcode(raw.subcode, raw.data).map_err(|e| e.into_de_error::<D>())?,
      ),
      _ if raw.code == *T::code() => RequestType::Local(
        T::from_subcode(raw.subcode, raw.data).map_err(|e| e.into_de_error::<D>())?,
      ),
      _ => {
        return Err(serde::de::Error::custom(
          PrimaryDeserializeError::UnmatchedCode(raw.code),
        ))
      }
    };
    Ok(Self {
      error_type: result,
      message: raw.message,
    })
  }
}

impl<T: Primary + PrimaryCreator> serde::Serialize for RequestType<T> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      Self::InvalidRequest(data) => Serializable::new(data).serialize(serializer),
      Self::Local(n) => Serializable::new(n).serialize(serializer),
      Self::Server(n) => Serializable::new(n).serialize(serializer),
    }
  }
}

pub struct Serializable<'a, T>(&'a T);

impl<'a, T: Primary + PrimaryCreator> Serializable<'a, T> {
  pub const fn new(value: &'a T) -> Self {
    Self(value)
  }
}

impl<'a, T: Primary + PrimaryCreator> serde::Serialize for Serializable<'a, T> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut len = 3;
    let subcode = match self.0.subcode() {
      Some(n) => {
        len += 1;
        Some(n)
      }
      None => None,
    };
    let data = match self.0.data().map_err(serde::ser::Error::custom)? {
      Some(d) => {
        len += 1;
        Some(d)
      }
      None => None,
    };

    let mut map = serializer.serialize_map(Some(len))?;
    map.serialize_entry("code", T::code())?;
    if let Some(subcode) = subcode {
      map.serialize_entry("subcode", subcode)?;
    }
    map.serialize_entry("message", &self.0.message())?;

    if let Some(data) = data {
      map.serialize_entry("data", &data)?;
    }

    map.end()
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explicit<T: PrimaryCreator> {
  pub data: T,
  pub message: String,
}

impl<'de, T: PrimaryCreator> serde::Deserialize<'de> for Explicit<T> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    struct Visitor<T>(PhantomData<T>);

    #[derive(Debug, Deserialize)]
    #[serde(field_identifier, rename_all = "snake_case")]
    enum Field {
      Code,
      Subcode,
      Message,
      Data,
    }

    impl<'de, T: PrimaryCreator> serde::de::Visitor<'de> for Visitor<T> {
      type Value = (T, String);

      fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "error data for {} type", T::name())
      }

      fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
      where
        A: serde::de::MapAccess<'de>,
      {
        let mut field_code: Option<u32> = None;
        let mut field_subcode: Option<u32> = None;
        let mut field_message: Option<String> = None;
        let mut field_data: Option<serde_value::Value> = None;

        while let Some(field) = map.next_key::<Field>()? {
          match field {
            Field::Code => {
              if field_code.is_some() {
                return Err(serde::de::Error::duplicate_field("code"));
              }
              field_code = Some(map.next_value()?);
            }
            Field::Subcode => {
              if field_subcode.is_some() {
                return Err(serde::de::Error::duplicate_field("subcode"));
              }
              field_subcode = Some(map.next_value()?);
            }
            Field::Message => {
              if field_message.is_some() {
                return Err(serde::de::Error::duplicate_field("message"));
              }
              field_message = Some(map.next_value()?);
            }
            Field::Data => {
              if field_data.is_some() {
                return Err(serde::de::Error::duplicate_field("data"));
              }
              field_data = Some(map.next_value()?);
            }
          }
        }

        let code = field_code.ok_or_else(|| serde::de::Error::missing_field("code"))?;

        if code != *T::code() {
          return Err(serde::de::Error::custom(format!(
            "expected code {:?}; got {code:?}",
            T::code()
          )));
        }

        let field_message =
          field_message.ok_or_else(|| serde::de::Error::missing_field("message"))?;

        T::from_subcode(field_subcode, field_data)
          .map(|v| (v, field_message))
          .map_err(|e| e.into_map_error::<A>())
      }
    }

    deserializer
      .deserialize_map(Visitor::<T>(PhantomData))
      .map(|(data, message)| Self { data, message })
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Untyped {
  pub code: u32,
  pub subcode: Option<u32>,
  pub message: String,
  pub data: Option<serde_value::Value>,
}

impl Untyped {
  pub fn deserialize_into<T: PrimaryCreator>(self) -> Result<(T, String), PrimaryDeserializeError> {
    if self.code == *T::code() {
      return Err(PrimaryDeserializeError::UnmatchedCode(self.code));
    }
    T::from_subcode(self.subcode, self.data).map(|v| (v, self.message))
  }
}

#[derive(Debug, Error)]
pub enum PrimaryDeserializeError {
  #[error("{0}")]
  Custom(serde_value::DeserializerError),
  #[error("invalid subcode of {0}; got {1:?}")]
  InvalidSubcode(&'static str, u32),
  #[error("missing data")]
  MissingData,
  #[error("missing subcode")]
  MissingSubcode,
  #[error("unexpected error code {0:?}")]
  UnmatchedCode(u32),
}

impl PrimaryDeserializeError {
  fn into_de_error<'de, D: serde::de::Deserializer<'de>>(self) -> D::Error {
    use serde::de::Error;
    match &self {
      Self::Custom(n) => Error::custom(n),
      Self::MissingData => Error::missing_field("data"),
      Self::MissingSubcode => Error::missing_field("subcode"),
      _ => Error::custom(self),
    }
  }

  fn into_map_error<'de, D: serde::de::MapAccess<'de>>(self) -> D::Error {
    use serde::de::Error;
    match &self {
      Self::Custom(n) => Error::custom(n),
      Self::MissingData => Error::missing_field("data"),
      Self::MissingSubcode => Error::missing_field("subcode"),
      _ => Error::custom(self),
    }
  }
}

pub trait PrimaryCreator {
  fn name() -> &'static str;
  fn code() -> &'static u32;

  #[doc(hidden)]
  fn from_subcode(
    subcode: Option<u32>,
    value: Option<serde_value::Value>,
  ) -> Result<Self, PrimaryDeserializeError>
  where
    Self: Sized;
}

pub trait Primary: Debug + Display + Send + Sync + 'static {
  fn subcode(&self) -> Option<&'static u32>;
  fn message(&self) -> Cow<'static, str>;

  /// Gets the additional error data serialized in [`serde_value::Value`].
  fn data(&self) -> Result<Option<serde_value::Value>, serde_value::SerializerError> {
    Ok(None)
  }
}

pub trait Tertiary {
  fn message(&self) -> Cow<'static, str>;
}
