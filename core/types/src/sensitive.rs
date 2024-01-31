use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Debug, Display};
use std::ops::Deref;

/// Keeps the raw sensitive data in memory but it cannot be
/// accidentally leaked through the console or logs.
///
/// If `server` feature is disabled, this type is that directly
/// referred to the generic argument.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize,
)]
#[serde(transparent)]
pub struct Sensitive<T>(T);

impl<T> Sensitive<T> {
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Debug for Sensitive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("<hidden>").finish()
    }
}

impl<T> Display for Sensitive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("<hidden>").finish()
    }
}

impl<T> AsRef<T> for Sensitive<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: Deref> Sensitive<T> {
    #[must_use]
    pub fn as_deref(&self) -> Sensitive<&T::Target> {
        Sensitive(self.0.deref())
    }
}

impl<T: Deref> Sensitive<Option<T>> {
    #[must_use]
    pub fn as_opt_deref(&self) -> Sensitive<Option<&T::Target>> {
        Sensitive(self.0.as_deref())
    }
}

impl<T: AsRef<str>> Sensitive<T> {
    #[must_use]
    pub fn into_string(self) -> String {
        self.0.as_ref().to_string()
    }
}

impl<T: AsRef<str>> Sensitive<Option<T>> {
    #[must_use]
    pub fn into_opt_string(&self) -> Option<String> {
        self.0.as_ref().map(|v| v.as_ref().to_string())
    }
}

impl<T: AsRef<str>> Sensitive<T> {
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl<T> From<T> for Sensitive<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> std::borrow::Borrow<T> for Sensitive<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl std::borrow::Borrow<str> for Sensitive<String> {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl<'a, T: AsRef<str> + 'a> From<&'a Sensitive<T>> for Cow<'a, str> {
    fn from(value: &'a Sensitive<T>) -> Self {
        Cow::Borrowed(value.0.as_ref())
    }
}

impl AsRef<[u8]> for Sensitive<String> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<[u8]> for Sensitive<Vec<u8>> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::Sensitive;
    use serde::{Deserialize, Serialize};
    use serde_test::Token;

    #[test]
    fn test_serde_impl() {
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Person {
            pub name: String,
            pub age: u32,
        }

        let person = Person { name: "memo".into(), age: 99 };
        serde_test::assert_tokens(
            &person,
            &[
                Token::Struct { name: "Person", len: 2 },
                Token::Str("name"),
                Token::Str("memo"),
                Token::Str("age"),
                Token::U32(99),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_fmt() {
        let value = Sensitive::new("hello");
        assert_eq!(value.to_string(), "<hidden>");
        assert_eq!(format!("{value:?}"), "<hidden>");
    }
}
