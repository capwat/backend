use capwat_types::Sensitive;
use serde::Deserialize;
use std::num::{NonZeroU32, NonZeroU64};
use std::time::Duration;
use validator::Validate;

mod defaults;

#[derive(Debug, Deserialize, Validate)]
pub struct Database {
  #[validate(nested)]
  pub(crate) primary: DatabasePool,
  #[validate(nested, optional)]
  pub(crate) replica: Option<DatabasePool>,
  #[serde(default = "defaults::default_enforce_tls")]
  pub(crate) enforce_tls: bool,
  #[serde(default = "defaults::default_pool_timeout_secs")]
  pub(crate) timeout_secs: NonZeroU64,
}

impl Database {
  #[must_use]
  pub const fn primary(&self) -> &DatabasePool {
    &self.primary
  }

  #[must_use]
  pub const fn replica(&self) -> Option<&DatabasePool> {
    self.replica.as_ref()
  }

  #[must_use]
  pub const fn enforces_tls(&self) -> bool {
    self.enforce_tls
  }

  #[must_use]
  pub const fn timeout(&self) -> Duration {
    Duration::from_secs(self.timeout_secs.get())
  }

  #[must_use]
  pub const fn timeout_secs(&self) -> u64 {
    self.timeout_secs.get()
  }
}

#[derive(Debug, Deserialize, Validate)]
pub struct DatabasePool {
  pub(crate) readonly: bool,
  pub(crate) min_idle: Option<NonZeroU32>,
  #[serde(default = "defaults::default_pool_size")]
  pub(crate) pool_size: NonZeroU32,
  #[validate(
    with = "validate_pg_url",
    error = "Invalid Postgres connection URL"
  )]
  pub(crate) url: Sensitive<String>,
}

impl DatabasePool {
  #[must_use]
  pub const fn readonly(&self) -> bool {
    self.readonly
  }

  #[must_use]
  pub const fn min_idle(&self) -> Option<u32> {
    match self.min_idle {
      Some(v) => Some(v.get()),
      None => None,
    }
  }

  #[must_use]
  pub const fn size(&self) -> u32 {
    self.pool_size.get()
  }

  #[must_use]
  pub const fn connection_url(&self) -> &Sensitive<String> {
    &self.url
  }
}

fn validate_pg_url(url: &str) -> bool {
  let mut accepted = false;
  if let Ok(url) = url::Url::parse(url) {
    accepted =
      url.as_str().starts_with("postgres://") && url.scheme() == "postgres";
  }
  accepted
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::hint::black_box;

  #[test]
  fn test_consts_not_crashing() {
    black_box(defaults::default_pool_size().get());
    black_box(defaults::default_pool_timeout_secs().get());
  }

  #[test]
  fn test_validate_pg_url() {
    assert!(validate_pg_url("postgres://hello.world"));
    assert!(!validate_pg_url("hello.world"));
  }
}
