use fundu::DurationParser;
use serde_with::{DeserializeAs, SerializeAs};
use std::time::Duration as StdDuration;

pub struct AsHumanDuration;

struct OptionalStdVisitor;

impl<'de> serde::de::Visitor<'de> for OptionalStdVisitor {
    type Value = Option<StdDuration>;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("human duration")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(None)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use fundu::TimeUnit;
        use serde::de::Error as DeError;

        const PARSER: DurationParser<'static> = DurationParser::builder()
            .time_units(&[
                TimeUnit::MilliSecond,
                TimeUnit::Second,
                TimeUnit::Minute,
                TimeUnit::Hour,
                TimeUnit::Day,
            ])
            .allow_time_unit_delimiter()
            .disable_exponent()
            .build();

        let parsed = PARSER.parse(v).map_err(DeError::custom)?;
        let value = StdDuration::try_from(parsed).map_err(DeError::custom)?;
        Ok(Some(value))
    }
}

struct StdVisitor;

impl<'de> serde::de::Visitor<'de> for StdVisitor {
    type Value = StdDuration;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("human duration")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use fundu::TimeUnit;
        use serde::de::Error as DeError;

        const PARSER: DurationParser<'static> = DurationParser::builder()
            .time_units(&[
                TimeUnit::MilliSecond,
                TimeUnit::Second,
                TimeUnit::Minute,
                TimeUnit::Hour,
                TimeUnit::Day,
            ])
            .allow_time_unit_delimiter()
            .disable_exponent()
            .build();

        let parsed = PARSER.parse(v).map_err(DeError::custom)?;
        StdDuration::try_from(parsed).map_err(DeError::custom)
    }
}

impl<'de> DeserializeAs<'de, StdDuration> for AsHumanDuration {
    fn deserialize_as<D>(deserializer: D) -> Result<StdDuration, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(StdVisitor)
    }
}

impl<'de> DeserializeAs<'de, Option<StdDuration>> for AsHumanDuration {
    fn deserialize_as<D>(deserializer: D) -> Result<Option<StdDuration>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(OptionalStdVisitor)
    }
}

impl SerializeAs<StdDuration> for AsHumanDuration {
    fn serialize_as<S>(source: &StdDuration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let duration: fundu::Duration = (*source).into();
        serializer.collect_str(&duration.to_string())
    }
}
