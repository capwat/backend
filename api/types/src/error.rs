use serde::{Deserialize, Serialize};
use std::hash::Hash;
#[cfg(feature = "server")]
use strum::Display;

// TODO: Check whether we need to compress this down up to 24 bytes
#[derive(Debug, Clone)]
#[must_use]
pub struct Error {
    pub category: ErrorCategory,
    pub message: Option<String>,
}

impl Error {
    pub fn new(category: ErrorCategory) -> Self {
        Self {
            category,
            message: None,
        }
    }

    pub fn message(self, message: impl Into<String>) -> Self {
        Self {
            message: Some(message.into()),
            ..self
        }
    }
}

impl Error {
    #[inline]
    #[must_use]
    pub fn code(&self) -> &str {
        self.category.code()
    }

    #[inline]
    #[must_use]
    pub fn subcode(&self) -> Option<&str> {
        self.category.subcode()
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.category == other.category
    }
}

impl Eq for Error {}

impl Hash for Error {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.category.hash(state);
    }
}

make_error_type! {
    #[cfg_attr(feature = "server", derive(Display, Default))]
    pub enum ErrorCategory {
        #[cfg_attr(feature = "server", default)]
        #[cfg_attr(feature = "server", strum(serialize = "Unknown error"))]
        Unknown,
        #[cfg_attr(feature = "server", strum(serialize = "Attempt to access while in read-only mode"))]
        ReadonlyMode,
        InvalidRequest,

        UserNotFound,
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct OtherError {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subcode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

macro_rules! make_error_type {
    {
        $( #[$Meta:meta] )*
        $Vis:vis enum $Name:ident {
            $(
                $(#[$VariantMeta:meta] )*
                $Variant:ident $(( $( $tt:tt )* ))?,
            )*
        }
    } => (paste::paste! {
        $( #[$Meta] )*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        $Vis enum $Name {
            $(
                $( #[$VariantMeta] )*
                $Variant $( ( $( $tt )* ) )?,
            )*
            Other(Box<OtherError>),
        }

        impl $Name {
            #[must_use]
            pub fn code(&self) -> &str {
                match self {
                    $( make_error_type!(variant: Self::$Variant $(, has_fields $($tt)*)?) => stringify!( [ <$Variant:snake:upper> ] ), )*
                    Self::Other(n) => &n.code,
                }
            }

            #[must_use]
            pub fn subcode(&self) -> Option<&str> {
                match self {
                    Self::Other(n) => n.subcode.as_deref(),
                    _ => None,
                }
            }
        }

        const _IMPL_DESERIALIZE: () = {
            make_error_type!(impl_deserialize: $Name, $( $Variant $( ( $( $tt )* ) )?, )*);
        };
        const _IMPL_SERIALIZE: () = {
            make_error_type!(impl_serialize: $Name, $( $Variant $( ( $( $tt )* ) )?, )*);
        };
    });
    // This is to deal with this Rust error:
    // "attempted to repeat an expression containing no syntax variables matched as repeating at this depth"
    (variant: $Name:ident::$Variant:ident include_data, has_fields $($tt:tt)*) => {
        // assuming that every error type can only have up to one field
        $Name::$Variant(data)
    };
    (variant: $Name:ident::$Variant:ident include_data) => {
        $Name::$Variant
    };
    (variant: $Name:ident::$Variant:ident, has_fields $($tt:tt)*) => {
        $Name::$Variant(..)
    };
    (variant: $Name:ident::$Variant:ident) => {
        $Name::$Variant
    };
    (impl_serialize: $Name:ident, $(
        $Variant:ident $(( $( $tt:tt )* ))?,
    )*) => (
        // Ordering of fields:
        // { "code": "LOGIN_FAILED", "subcode": "2FA_CHALLENGE", "message": "You've challenged to perfom 2FA auth..." }
        #[allow(non_local_definitions)]
        impl Serialize for Error {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut map = serializer.serialize_map(None)?;
                <_ as ::serde::ser::SerializeMap>::serialize_entry(&mut map, "code", self.code())?;
                if let ::std::option::Option::Some(subcode) = self.subcode() {
                    <_ as ::serde::ser::SerializeMap>::serialize_entry(&mut map, "subcode", subcode)?;
                }
                if let ::std::option::Option::Some(message) = self.message.as_ref() {
                    <_ as ::serde::ser::SerializeMap>::serialize_entry(&mut map, "message", message)?;
                }
                match &self.category {
                    $( make_error_type!(variant: $Name::$Variant include_data $(, has_fields $($tt)*)?) =>
                        make_error_type!(impl_serialize_variant_impl $(: has_fields $($tt)*)?), )*
                    $Name::Other(inner) => {
                        if let Some(data) = inner.data.as_ref() {
                            <_ as ::serde::ser::SerializeMap>::serialize_entry(&mut map, "data", data)?;
                        }
                    }
                }
                <_ as ::serde::ser::SerializeMap>::end(map)
            }
        }
    );
    // This is to deal with this Rust error:
    // "attempted to repeat an expression containing no syntax variables matched as repeating at this depth"
    (impl_serialize_variant_impl: has_fields $($tt:tt)*) => {
        // assuming that every error type can only have up to one field
        <_ as ::serde::ser::SerializeMap>::serialize_entry(&mut map, "data", data)?;
    };
    // do nothing, really
    (impl_serialize_variant_impl) => {{}};
    (impl_deserialize: $Name:ident, $(
        $Variant:ident $(( $( $tt:tt )* ))?,
    )*) => (paste::paste! {
        struct ErrorVisitor;

        impl<'de> ::serde::de::Visitor<'de> for ErrorVisitor {
            type Value = Error;

            fn expecting(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str("Capwat error")
            }

            fn visit_map<A>(self, mut map: A) -> ::std::result::Result<Self::Value, A::Error>
            where
                A: ::serde::de::MapAccess<'de>,
            {
                #[derive(Debug, ::serde::Deserialize)]
                #[serde(field_identifier, rename_all = "snake_case")]
                enum Field {
                    Code,
                    Subcode,
                    Message,
                    Data,
                    #[serde(other)]
                    Other,
                }

                let mut code: ::std::option::Option<String> = None;
                let mut subcode: ::std::option::Option<String> = None;
                let mut message: ::std::option::Option<String> = None;
                // we're expecting it's from a JSON data
                let mut data: ::std::option::Option<serde_json::Value> = None;

                while let Some(field) = map.next_key::<Field>()? {
                    match field {
                        Field::Code => {
                            code = Some(map.next_value()?);
                        }
                        Field::Subcode => {
                            subcode = map.next_value()?;
                        }
                        Field::Message => {
                            message = map.next_value()?;
                        }
                        Field::Data => {
                            data = map.next_value()?;
                        }
                        Field::Other => {
                            map.next_value::<::serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let code = code.ok_or_else(|| ::serde::de::Error::missing_field("code"))?;
                match code.as_str() {
                    $(
                        stringify!( [ <$Variant:snake:upper> ] ) => Ok(Error {
                            category: make_error_type!(impl_deserialize_field: $Name, $Variant $(, has_fields $($tt)*)?),
                            message,
                        }),
                    )*
                    _ => Ok(Error {
                        category: $Name::Other(::std::boxed::Box::new(OtherError {
                            code,
                            subcode,
                            data,
                        })),
                        message,
                    }),
                }
            }
        }

        #[allow(non_local_definitions)]
        impl<'de> ::serde::de::Deserialize<'de> for Error {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                <_ as ::serde::Deserializer<'de>>::deserialize_map(deserializer, ErrorVisitor)
            }
        }
    });
    // This is to deal with this Rust error:
    // "attempted to repeat an expression containing no syntax variables matched as repeating at this depth"
    (impl_deserialize_field: $Name:ident, $Variant:ident, has_fields $($tt:tt)*) => {
        todo!()
    };
    (impl_deserialize_field: $Name:ident, $Variant:ident) => {
        $Name::$Variant
    };
}
use make_error_type;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use serde_test::{assert_ser_tokens, Token};
    use static_assertions::{assert_eq_size, assert_impl_all};

    assert_eq_size!(ErrorCategory, u128);

    assert_impl_all!(
        Error: std::fmt::Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        Send,
        Sync,
    );
    assert_impl_all!(
        ErrorCategory: std::fmt::Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        Send,
        Sync,
    );

    macro_rules! make_error {
    ($message:literal, $( $tt:tt )*) => (Error {
        category: ErrorCategory::$( $tt )*,
        message: Some($message.into()),
    });
    (None, $( $tt:tt )*) => (Error {
        category: ErrorCategory::$( $tt )*,
        message: None,
    });
}

    #[test]
    fn test_code() {
        assert_eq!(ErrorCategory::ReadonlyMode.code(), "READONLY_MODE");

        let unknown = ErrorCategory::Other(Box::new(OtherError {
            code: "HI_ERROR".into(),
            subcode: None,
            data: None,
        }));
        assert_eq!(unknown.code(), "HI_ERROR");
    }

    #[test]
    fn test_serialize_unknown() {
        let error = make_error!(
            "I have no idea about this error",
            Other(Box::new(OtherError {
                code: "DESTROYED".into(),
                subcode: Some("BOMBED".into()),
                data: Some(json!({ "time": "12:45 PM" }))
            }))
        );
        assert_ser_tokens(
            &error,
            &[
                Token::Map { len: None },
                Token::Str("code"),
                Token::Str("DESTROYED"),
                Token::Str("subcode"),
                Token::Str("BOMBED"),
                Token::Str("message"),
                Token::Str("I have no idea about this error"),
                Token::Str("data"),
                Token::Map { len: Some(1) },
                Token::Str("time"),
                Token::Str("12:45 PM"),
                Token::MapEnd,
                Token::MapEnd,
            ],
        );

        let error = make_error!(
            None,
            Other(Box::new(OtherError {
                code: "DESTROYED".into(),
                subcode: Some("BOMBED".into()),
                data: Some(json!({ "time": "12:45 PM" }))
            }))
        );
        assert_ser_tokens(
            &error,
            &[
                Token::Map { len: None },
                Token::Str("code"),
                Token::Str("DESTROYED"),
                Token::Str("subcode"),
                Token::Str("BOMBED"),
                Token::Str("data"),
                Token::Map { len: Some(1) },
                Token::Str("time"),
                Token::Str("12:45 PM"),
                Token::MapEnd,
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn test_serialize_known_type_without_data() {
        let error = make_error!("Hello", ReadonlyMode);
        assert_ser_tokens(
            &error,
            &[
                Token::Map { len: None },
                Token::Str("code"),
                // serde-test won't let me use dynamically values which
                // it includes `error.code()`
                Token::Str("READONLY_MODE"),
                Token::Str("message"),
                Token::Str("Hello"),
                Token::MapEnd,
            ],
        );

        let error = make_error!(None, ReadonlyMode);
        assert_ser_tokens(
            &error,
            &[
                Token::Map { len: None },
                Token::Str("code"),
                // serde-test won't let me use dynamically values which
                // it includes `error.code()`
                Token::Str("READONLY_MODE"),
                Token::MapEnd,
            ],
        );
    }
}
