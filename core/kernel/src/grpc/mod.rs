mod impls;
pub mod proto;

use error_stack::{Result, ResultExt};
use thiserror::Error;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};

use crate::config::GrpcConfig;

#[derive(Debug, Clone)]
pub struct GrpcClient(Channel);

#[derive(Debug, Error)]
#[error("Failed to initialize gRPC client")]
pub struct GrpcClientInitError;

impl GrpcClient {
  pub fn new(cfg: &GrpcConfig) -> Result<Self, GrpcClientInitError> {
    let mut endpoints = Vec::new();

    // TODO: Support for custom certificates.
    //
    // Right now, you'll need to install your custom certificate
    // in your operating system (or any)'s certificate store.
    let tls = ClientTlsConfig::new();

    for addr in cfg.addresses() {
      endpoints.push(
        Endpoint::from_shared(addr.as_str().to_string())
          .change_context(GrpcClientInitError)?
          .timeout(cfg.timeout())
          .tls_config(tls.clone())
          .change_context(GrpcClientInitError)?
          .connect_timeout(cfg.timeout()),
      );
    }

    Ok(Self(Channel::balance_list(endpoints.into_iter())))
  }

  #[must_use]
  pub fn get_channel(&self) -> Channel {
    self.0.clone()
  }
}
