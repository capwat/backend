use error_stack::{Report, Result, ResultExt};
use sensitive::Sensitive;
use serde::Deserialize;
use validator::Validate;

use super::ParseError;
use crate::util::{figment::FigmentErrorAttachable, validator::IntoValidatorReport};

#[derive(Debug, Deserialize, Validate)]
pub struct Server {
    #[validate(nested)]
    pub db: super::Database,
    #[validate(length(min = 12, max = 1024), error = "Invalid JWT secret key")]
    pub jwt_secret: Sensitive<String>,
}

impl Server {
    pub fn load() -> Result<Self, ParseError> {
        dotenvy::dotenv().ok();

        let config = Self::figment()
            .extract::<Self>()
            .map_err(|e| Report::new(ParseError).attach_figment_error(e))?;

        config
            .validate()
            .into_validator_report()
            .change_context(ParseError)?;

        Ok(config)
    }
}

impl Server {
    const DEFAULT_CONFIG_FILE: &str = "whim.yml";

    /// Creates a default [`Figment`] object to load server
    /// configuration. This function is there for implementing
    /// [`Server::default`] and testing.
    pub(crate) fn figment() -> figment::Figment {
        use figment::{
            providers::{Env, Format, Yaml},
            Figment,
        };

        Figment::new()
            .merge(Yaml::file(Self::DEFAULT_CONFIG_FILE))
            // One big con about figment (env provider to be specific) especially
            // these fields with underscore in it.
            .merge(Env::prefixed("WHIM_").map(|v| match v.as_str() {
                "DB_PRIMARY_MIN_IDLE" => "db.primary.min_idle".into(),
                "DB_PRIMARY_POOL_SIZE" => "db.primary.pool_size".into(),

                "DB_REPLICA_MIN_IDLE" => "db.replica.min_idle".into(),
                "DB_REPLICA_POOL_SIZE" => "db.replica.pool_size".into(),

                "DB_ENFORCE_TLS" => "db.enforce_tls".into(),
                "DB_TIMEOUT_SECS" => "db.timeout_secs".into(),

                _ => v.as_str().replace("_", ".").into(),
            }))
            // Environment variable aliases
            .merge(Env::raw().map(|v| match v.as_str() {
                "DATABASE_URL" => "db.primary.url".into(),
                _ => v.into(),
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use figment::Jail;
    use std::num::{NonZeroU32, NonZeroU64};

    #[test]
    fn env_aliases() {
        Jail::expect_with(|jail| {
            jail.set_env("DATABASE_URL", "hello world!");

            jail.set_env("WHIM_DB_PRIMARY_MIN_IDLE", "100");
            jail.set_env("WHIM_DB_PRIMARY_POOL_SIZE", "100");

            jail.set_env("WHIM_DB_REPLICA_URL", "required");
            jail.set_env("WHIM_DB_REPLICA_MIN_IDLE", "589");
            jail.set_env("WHIM_DB_REPLICA_POOL_SIZE", "589");

            jail.set_env("WHIM_DB_ENFORCE_TLS", "false");
            jail.set_env("WHIM_DB_TIMEOUT_SECS", "3030");

            let config: Server = Server::figment().extract()?;
            assert_eq!(config.db.primary.url.as_str(), "hello world!");
            assert_eq!(
                config.db.primary.min_idle.unwrap(),
                NonZeroU32::new(100).unwrap()
            );
            assert_eq!(config.db.primary.pool_size, NonZeroU32::new(100).unwrap());
            assert_eq!(
                config.db.replica.as_ref().unwrap().min_idle.unwrap(),
                NonZeroU32::new(589).unwrap()
            );
            assert_eq!(
                config.db.replica.as_ref().unwrap().pool_size,
                NonZeroU32::new(589).unwrap()
            );

            assert_eq!(config.db.enforce_tls, false);
            assert_eq!(config.db.timeout_secs, NonZeroU64::new(3030).unwrap());

            Ok(())
        });
    }
}
