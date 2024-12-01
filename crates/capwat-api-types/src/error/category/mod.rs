use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use strum::Display;

capwat_macros::define_error_category! {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    #[cfg_attr(feature = "server", derive(Display))]
    {
        /// We don't know what is the cause of this error but the error we have in
        /// our server is reported to the developers.
        Unknown,

        /// Capwat instance is currently in read-only mode and should not perform
        /// any write operations at the moment.
        ReadonlyMode,
        InvalidRequest,

        /// `Outage` can mean that one service is down and cannot perform the action as
        /// intended to the user such as when the database of a Capwat instance is down.
        Outage,

        /// This variant allows to inform users that the Capwat instance is closed
        /// likely because the administrator closes down the entire website likely
        /// from maintenance to user safety.
        InstanceClosed,

        /// It seems like you don't have email address yet but the Capwat instance
        /// requires you to do that. Please comply before using the service.
        NoEmailAddress,

        /// You haven't verified your email address yet. Please do so before you
        /// operate the entire API.
        EmailVerificationRequired,

        /// This variant shows the possiblities of why logging in as a user
        /// failed in the first place.
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        LoginUserFailed {
            InvalidCredientials,
        },

        /// This variant shows the possiblities of why user registration failed
        /// in the first place. It contains mostly user input only.
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        RegisterUserFailed {
            /// User registration is closed!
            Closed,

            UsernameTaken,
            EmailTaken,
            EmailRequired,

            PublicKeyNotUnique,
            SaltNotUnique,
            UnsupportedKeyAlgorithm,
        },

        // CaptchaRequired(CaptchaInfo)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OtherError {
    pub code: ErrorCode,
    pub data: Option<serde_json::Value>,
}

pub struct ErrorDeserializer<'a> {
    code: Option<&'a str>,
    subcode: Option<&'a str>,
}

impl<'a> ErrorDeserializer<'a> {
    pub fn from_json(input: &'a str) -> Self {
        Self {
            code: Self::find_str_value(input, "code"),
            subcode: Self::find_str_value(input, "subcode"),
        }
    }

    fn find_str_value(input: &'a str, field: &'static str) -> Option<&'a str> {
        let json_field = format!("\"{field}\":");
        let from = input.find(&json_field)? + json_field.len();

        let buffer = input[from..].trim_start();
        let buffer = buffer
            .chars()
            .next()
            .and_then(|v| (v == '"').then(|| &buffer[1..]))?;

        let mut processed = buffer;
        loop {
            let pos = processed.find(&['"', '\\'] as &[_])?;
            let stripped = &processed[pos..];

            // scan the entire control character or stuff :)
            if let Some(mut stripped) = stripped.strip_prefix('\\') {
                let control_character = stripped.chars().next()?;
                match control_character {
                    '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' => {
                        processed = &stripped[1..];
                    }
                    'u' => {
                        stripped = &stripped[1..];
                        for _ in 0..4 {
                            let char = stripped.chars().next()?;
                            if !char.is_ascii_hexdigit() {
                                return None;
                            }
                            stripped = &stripped[1..];
                        }
                        processed = stripped;
                    }
                    _ => return None,
                }
            } else {
                let left = stripped.len();
                let raw_length = buffer.len() - left;
                return Some(&buffer[..raw_length]);
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct CaptchaInfo {
    pub captcha_token: String,
}
