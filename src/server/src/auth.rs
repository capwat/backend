pub mod jwt {
    use bitflags::bitflags;
    use capwat_error::Result;
    use capwat_model::User;
    use chrono::{DateTime, TimeDelta, Utc};
    use serde::{Deserialize, Serialize};

    use crate::app::auth::{DecodeJwtError, EncodeJwtError, JwtIssuer};
    use crate::App;

    bitflags! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct Scope: u64 {
            const APPLICATION = 1 << 1;
        }
    }

    impl Scope {
        #[must_use]
        pub fn has_permission(&self, reqs: Scope) -> bool {
            self.contains(Self::APPLICATION) || self.intersects(reqs)
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct LoginClaims {
        pub nbf: i64,
        pub exp: i64,
        pub iss: String,
        pub sub: i64,

        pub name: String,
        pub scope: Scope,
    }

    impl LoginClaims {
        pub fn generate(
            app: &App,
            user: &User,
            now: Option<DateTime<Utc>>,
            scopes: Option<Scope>,
        ) -> Self {
            let now = now.unwrap_or_else(Utc::now);
            Self {
                nbf: now.timestamp(),
                exp: (now + TimeDelta::days(1)).timestamp(),
                iss: JwtIssuer::Login.to_string(app),
                sub: user.id.0,
                name: user.name.clone(),
                scope: scopes.unwrap_or_else(|| Scope::APPLICATION),
            }
        }

        pub fn decode(app: &App, token: &str) -> Result<Self, DecodeJwtError> {
            app.decode_jwt(token, &JwtIssuer::Login)
        }

        pub fn encode(&self, app: &App) -> Result<String, EncodeJwtError> {
            app.encode_to_jwt(self)
        }
    }

    impl<'de> serde::de::Deserialize<'de> for Scope {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = Scope;

                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str("Capwat JWT scopes")
                }

                fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(Scope::from_bits_truncate(v))
                }

                fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    v.parse::<u64>()
                        .map_err(serde::de::Error::custom)
                        .and_then(|v| self.visit_u64(v))
                }
            }

            deserializer.deserialize_any(Visitor)
        }
    }

    impl Serialize for Scope {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.collect_str(&self.bits())
        }
    }
}
