use capwat_types_derive::Error;
use std::fmt::Display;

mod deserializer;
mod serialization;
mod traits;
mod unknown;

mod variants;
pub use variants::*;

pub(crate) use traits::*;

pub use deserializer::ErrorDeserializer;
pub use unknown::Unknown;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum Error {
    #[error(code = 1)]
    #[error(message = "Internal server occurred. Please try again later.")]
    Internal,
    #[error(code = 2)]
    #[error(
        message = "This service is currently in read only mode. Please try again later."
    )]
    ReadonlyMode,
    #[error(code = 3)]
    #[error(message = "Not authenticated")]
    NotAuthenticated,
    #[error(code = 4)]
    #[error(message = "Requested entry does not exists")]
    NotFound,
    #[error(code = 5)]
    LoginUser(Box<LoginUser>),
    #[error(unknown)]
    Unknown(Box<Unknown>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Internal => f.write_str("Failed to perform request"),
            Error::ReadonlyMode => {
                f.write_str("Attempted to write while in read only mode")
            },
            Error::NotAuthenticated => f.write_str(
                "Attempted to access resource while not authenticated",
            ),
            Error::NotFound => {
                f.write_str("Attempted to access non-existing resource")
            },
            Error::LoginUser(n) => Display::fmt(&n, f),
            Error::Unknown(info) => {
                write!(f, "Unknown({}", info.code)?;
                if let Some(subcode) = info.subcode {
                    write!(f, ":{subcode}")?;
                }
                write!(f, "): {}", info.message)
            },
        }
    }
}

impl Error {
    #[must_use]
    pub fn message(&self) -> String {
        struct MessageMaker<'a>(&'a Error);

        impl<'a> Display for MessageMaker<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0._make_message(f)
            }
        }

        MessageMaker(self).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, LoginUser};
    use serde_json::json;
    use serde_test::{assert_ser_tokens, Token};
    use static_assertions::{assert_eq_size, assert_impl_all};

    assert_eq_size!(Error, u128);
    assert_eq_size!(LoginUser, u64);

    assert_impl_all!(Error: std::fmt::Debug, std::fmt::Display, Clone,
      Send, Sync, serde::Serialize, PartialEq, Eq);

    #[test]
    fn test_serialization() {
        let internal = Error::Internal;
        assert_ser_tokens(
            &internal,
            &[
                Token::Map { len: Some(2) },
                Token::Str("code"),
                Token::U64(Error::INTERNAL_CODE),
                Token::Str("message"),
                Token::Str("Internal server occurred. Please try again later."),
                Token::MapEnd,
            ],
        );

        // Complex serialization
        let unknown = Error::Unknown(Box::new(super::Unknown {
            code: 1000,
            subcode: Some(200),
            message: "Hi!".into(),
            data: Some(json!({
                "name": "serde",
            })),
        }));
        assert_ser_tokens(
            &unknown,
            &[
                Token::Map { len: Some(4) },
                Token::Str("code"),
                Token::U64(1000),
                Token::Str("subcode"),
                Token::U64(200),
                Token::Str("message"),
                Token::Str("Hi!"),
                Token::Str("data"),
                Token::Some,
                Token::Map { len: Some(1) },
                Token::Str("name"),
                Token::Str("serde"),
                Token::MapEnd,
                Token::MapEnd,
            ],
        );
    }
}
