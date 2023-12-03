use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use std::fmt::Display;
use std::hash::Hash;
use std::num::NonZeroU64;
use std::ops::Deref;
use std::str::FromStr;
use thiserror::Error;

// Capwat epoch starts at November 18, 2023 at 07:52:48 AM in Manila time
const EPOCH: u64 = 1_700_265_168_293;
const TIMESTAMP_BITS_LEN: usize = 43; // It should last up to 278 years

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
  #[must_use]
  pub fn now() -> Self {
    Self(Utc::now())
  }

  pub fn from_timestamp(secs: i64) -> Result<Self, InvalidTimestamp> {
    let dt =
      NaiveDateTime::from_timestamp_opt(secs, 0).ok_or(InvalidTimestamp)?;
    Ok(Self(DateTime::from_naive_utc_and_offset(dt, Utc)))
  }

  pub fn parse(input: &str) -> Result<Self, ParseError> {
    DateTime::parse_from_rfc3339(input)
      .map(|v| Self(v.with_timezone(&Utc)))
      .map_err(ParseError)
  }

  #[allow(clippy::cast_possible_wrap)]
  pub(crate) fn from_snowflake(id: NonZeroU64) -> Self {
    let unix_timestamp = (id.get() >> (63 - TIMESTAMP_BITS_LEN)) + EPOCH;
    Self(Utc.timestamp_millis_opt(unix_timestamp as i64).unwrap())
  }

  #[must_use]
  pub fn timestamp(&self) -> i64 {
    self.0.timestamp()
  }
}

impl Hash for Timestamp {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.0.hash(state);
  }
}

impl<Tz: TimeZone> From<DateTime<Tz>> for Timestamp {
  fn from(dt: DateTime<Tz>) -> Self {
    Self(dt.with_timezone(&Utc))
  }
}

impl From<NaiveDateTime> for Timestamp {
  fn from(value: NaiveDateTime) -> Self {
    Self(value.and_utc())
  }
}

impl From<Timestamp> for NaiveDateTime {
  fn from(value: Timestamp) -> Self {
    value.0.naive_utc()
  }
}

impl Display for Timestamp {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = self.0.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    s.fmt(f)
  }
}

impl Deref for Timestamp {
  type Target = DateTime<Utc>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl FromStr for Timestamp {
  type Err = ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::parse(s)
  }
}

impl<'de> serde::Deserialize<'de> for Timestamp {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
      type Value = Timestamp;

      fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RFC 3339 timestamp")
      }

      fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        self.visit_str(&v)
      }

      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
      where
        E: serde::de::Error,
      {
        Timestamp::parse(v).map_err(serde::de::Error::custom)
      }
    }

    deserializer.deserialize_str(Visitor)
  }
}

impl serde::Serialize for Timestamp {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.collect_str(self)
  }
}

#[derive(Debug, Error)]
#[error("Invalid UNIX timestamp value")]
pub struct InvalidTimestamp;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct ParseError(chrono::ParseError);

impl From<ParseError> for chrono::ParseError {
  fn from(value: ParseError) -> Self {
    value.0
  }
}

impl Deref for ParseError {
  type Target = chrono::ParseError;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_test::Token;

  #[test]
  fn test_fmt_display_impl() {
    let timestamp = Timestamp::from_snowflake(1.try_into().unwrap());
    assert_eq!("2023-11-17T23:52:48.293Z", timestamp.to_string());
  }

  #[test]
  fn test_serde_impl() {
    let timestamp = Timestamp::from_snowflake(1.try_into().unwrap());
    serde_test::assert_tokens(
      &timestamp,
      &[Token::Str("2023-11-17T23:52:48.293Z")],
    );
  }

  #[test]
  fn test_from_snowflake() {
    let timestamp = Timestamp::from_snowflake(1.try_into().unwrap());
    assert_eq!("2023-11-17T23:52:48.293Z", timestamp.to_string());

    let timestamp = Timestamp::from_snowflake(
      (23_400_000_055 << (63 - TIMESTAMP_BITS_LEN)).try_into().unwrap(),
    );
    assert_eq!("2024-08-14T19:52:48.348Z", timestamp.to_string());
  }
}
