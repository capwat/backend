use std::borrow::Cow;
use std::str::FromStr;

use super::Error;
use crate::error::ErrorVisitor;

#[derive(Debug)]
pub struct ErrorDeserializer<'a> {
    c: u64,
    s: Option<u64>,
    m: Cow<'a, str>,
}

// Based from: https://github.com/twilight-rs/twilight/blob/main/twilight-model/src/gateway/event/gateway.rs#L56-L178
// Licensed under ISC (Internet Systems Consortium) license
impl<'a> ErrorDeserializer<'a> {
    #[must_use]
    pub fn new(code: u64, subcode: Option<u64>, message: Cow<'a, str>) -> Self {
        Self { c: code, s: subcode, m: message }
    }

    pub fn from_json(input: &'a str) -> Option<Self> {
        let code = Self::find_integer(input, r#""code":"#)?;
        let subcode = Self::find_integer(input, r#""subcode":"#);
        let message = Self::find_string(input, r#""message":"#)?;

        Some(Self { c: code, s: subcode, m: message })
    }

    #[must_use]
    pub fn into_owned(self) -> ErrorDeserializer<'static> {
        ErrorDeserializer {
            c: self.c,
            s: self.s,
            m: Cow::Owned(self.m.into_owned()),
        }
    }

    fn find_string(input: &'a str, key: &str) -> Option<Cow<'a, str>> {
        // We're going to search for the event type key from the start. Discord
        // always puts it at the front before the D key from some testing of
        // several hundred payloads.
        //
        // If we find it, add 4, since that's the length of what we're searching
        // for.
        let from = input.find(key)? + key.len();

        // Every valid string value in JSON must present a double quote
        // character to start with a new text until the last valid double
        // quote (excluding escaped characters).
        let trimmed = input.get(from..)?.trim_start();
        if trimmed.chars().next()? != '"' {
            return None;
        }

        // Start of the string value content
        let from = (input.len() - trimmed.len()) + 1;

        // Relative to the from position
        let mut to = 0;

        // This is to save allocation time if it has no escape
        // characters to deal with.
        let mut has_escape = false;

        while let Some(pos) = input[from..].get(to..)?.find('\\') {
            to += pos + 2;
            has_escape = true;
        }

        let to = to + input[from..][to..].find('"')?;

        // Try to parse the string with actual JSON deserializer
        if has_escape {
            let value = &input[(from - 1)..].get(..(to + 2))?;
            let value = serde_json::from_str::<String>(value).ok()?;
            Some(Cow::Owned(value))
        } else {
            let value = &input[from..].get(..to)?;
            Some(Cow::Borrowed(&value))
        }
    }

    fn find_integer<T: FromStr>(input: &'a str, key: &str) -> Option<T> {
        // Find the op key's position and then search for where the first
        // character that's not base 10 is. This'll give us the bytes with the
        // op which can be parsed.
        //
        // Add 5 at the end since that's the length of what we're finding
        let from = input.find(key)? + key.len();

        // Look for the first thing that isn't a base 10 digit or whitespace,
        // i.e. a comma (denoting another JSON field), curly brace (end of the
        // object), etc. This'll give us the op number, maybe with a little
        // whitespace.
        let to = input.get(from..)?.find(&[',', '}'] as &[_])?;
        // We might have some whitespace, so let's trim this.
        let clean = input.get(from..from + to)?.trim();

        T::from_str(clean).ok()
    }
}

impl<'a, 'de> serde::de::DeserializeSeed<'de> for ErrorDeserializer<'a> {
    type Value = Error;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(ErrorVisitor::new(self.c, self.s, self.m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_usage() {
        let deserializer =
            ErrorDeserializer::from_json(r#"{"code":25,"message":"Hi!"}"#)
                .unwrap();

        assert_eq!(25, deserializer.c);
        assert_eq!("Hi!", deserializer.m);

        let deserializer = ErrorDeserializer::from_json(
            r#"{"code":25,"subcode":10,"message":"Hi!"}"#,
        )
        .unwrap();

        assert_eq!(25, deserializer.c);
        assert_eq!(Some(10), deserializer.s);
        assert_eq!("Hi!", deserializer.m);
    }

    #[test]
    fn escaped_strings() {
        let deserializer =
            ErrorDeserializer::from_json(r#" "code": 20, "message":"Hi\"""#);
        assert!(deserializer.is_some());

        let deserializer = ErrorDeserializer::from_json(
            r#" "code": 20,"message":"Hi\\\"\"" "#,
        );
        assert!(deserializer.is_some());

        let deserializer =
            ErrorDeserializer::from_json(r#" "code": 20, "message":"Hi\"""#)
                .unwrap();

        assert_eq!("Hi\"", deserializer.m);

        let deserializer = ErrorDeserializer::from_json(
            r#" "code": 20, "message":"Hi\n\"\"""#,
        )
        .unwrap();
        assert_eq!("Hi\n\"\"", deserializer.m);
    }

    #[test]
    fn trailling_strings() {
        let deserializer = ErrorDeserializer::from_json(
            r#" "code": 20,
        "message":"Hi
        "#,
        );
        assert!(deserializer.is_none());

        let deserializer = ErrorDeserializer::from_json(
            r#" "code": 20,
        "message":"Hi\"
        "#,
        );
        assert!(deserializer.is_none());
    }
}
