use capwat_kernel::{
  entity::{
    id::{marker::UserMarker, Id},
    User,
  },
  services::{impl_dev::*, DataService},
};
use error_stack::{Result as StackResult, ResultExt};
use std::{str::FromStr, sync::Arc};
use thiserror::Error;
use tokio::sync::Mutex;
use tonic::{
  async_trait,
  transport::{Channel, Endpoint, Uri},
};

use crate::protobuf::{data_client::DataClient, GetUserByIdRequest};

#[derive(Debug)]
pub struct ClientLayer {
  client: Arc<Mutex<DataClient<Channel>>>,
}

#[derive(Debug, Error)]
pub enum ClientLayerError {
  #[error("Invalid endpoint while trying to connect to the data services")]
  InvalidEndpoint,
  #[error("Failed to connect to one of the endpoints")]
  FailedConnection,
}

impl ClientLayer {
  #[tracing::instrument]
  pub fn connect(endpoints: &[&str]) -> StackResult<Self, ClientLayerError> {
    let mut parsed_endpoints = Vec::new();
    for endpoint in endpoints {
      let uri = Uri::from_str(endpoint)
        .change_context(ClientLayerError::InvalidEndpoint)
        .attach_printable_lazy(|| format!("with endpoint: {endpoint}"))?;

      let endpoint = Endpoint::new(uri)
        .change_context(ClientLayerError::InvalidEndpoint)
        .attach_printable_lazy(|| format!("with endpoint: {endpoint}"))?
        .user_agent(concat!("Capwat-gRPC-Client/", env!("CARGO_PKG_VERSION")))
        .expect("should parse default static header");

      parsed_endpoints.push(endpoint);
    }

    let channel = Channel::balance_list(parsed_endpoints.into_iter());
    let layer = Self { client: Arc::new(Mutex::new(DataClient::new(channel))) };

    Ok(layer)
  }
}

#[async_trait]
impl DataService for ClientLayer {
  #[tracing::instrument]
  async fn find_user_by_id(
    &self,
    id: Id<UserMarker>,
  ) -> ServiceResult<Option<User>> {
    let mut client = self.client.lock().await;
    let response = client
      .get_user_by_id(GetUserByIdRequest { id: id.get() })
      .await?
      .into_inner()
      .user;

    if let Some(response) = response {
      let user = User {
        id,
        created_at: response.created_at.parse().into_capwat()?,
        name: response.name,
        email: response.email,
        display_name: response.display_name,
        password_hash: response.password_hash,
        updated_at: if let Some(updated_at) = response.updated_at {
          Some(updated_at.parse().into_capwat()?)
        } else {
          None
        },
      };
      Ok(Some(user))
    } else {
      Ok(None)
    }
  }
}
