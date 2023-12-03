use capwat_types::Sensitive;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use std::time::Duration;
use url::Url;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct GrpcConfig {
  #[validate(
    error = "One of gRPC address is invalid",
    with = "validate_rpc_addresses"
  )]
  addresses: Vec<Sensitive<String>>,
  #[serde(default = "default_timeout_secs")]
  timeout_secs: NonZeroU64,
}

impl GrpcConfig {
  pub fn addresses(&self) -> std::slice::Iter<'_, Sensitive<String>> {
    self.addresses.iter()
  }

  #[must_use]
  pub fn timeout(&self) -> Duration {
    Duration::from_secs(self.timeout_secs.get())
  }
}

fn default_timeout_secs() -> NonZeroU64 {
  const TEN_SECONDS: u64 = 10;
  NonZeroU64::new(TEN_SECONDS).expect("valid non-zero u64 passed")
}

fn validate_rpc_addresses(vec: &[Sensitive<String>]) -> bool {
  vec.iter().all(|v| {
    // gRPC uses HTTP under the hood
    Url::parse(v)
      .map(|v| {
        let scheme = v.scheme();
        let has_valid_scheme =
          scheme.is_empty() || matches!(scheme, "http" | "https");

        v.host().is_some() && has_valid_scheme
      })
      .unwrap_or_default()
  })
}
