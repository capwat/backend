use serde::Deserialize;
use std::fmt::Display;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoggingStyle {
    Compact,
    #[default]
    Full,
    Pretty,
    JSON,
}

impl Display for LoggingStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compact => f.write_str("compact"),
            Self::Full => f.write_str("full"),
            Self::Pretty => f.write_str("pretty"),
            Self::JSON => f.write_str("json"),
        }
    }
}

impl<'de> Deserialize<'de> for LoggingStyle {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = LoggingStyle;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("logging style")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                LoggingStyle::from_str(v).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[derive(Debug, Error)]
#[error("unknown {0:?} logging style")]
pub struct InvalidLoggingStyle(String);

impl FromStr for LoggingStyle {
    type Err = InvalidLoggingStyle;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "compact" => Ok(Self::Compact),
            "full" => Ok(Self::Full),
            "pretty" => Ok(Self::Pretty),
            "json" => Ok(Self::JSON),
            _ => Err(InvalidLoggingStyle(s)),
        }
    }
}
