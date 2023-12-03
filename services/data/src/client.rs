use async_trait::async_trait;
use capwat_kernel::config::GrpcConfig;
use capwat_kernel::drivers::{self, prelude::*};
use capwat_kernel::entity::{Secret, User};
use capwat_kernel::grpc::proto::data_client::DataClient;
use capwat_kernel::grpc::{GrpcClient, GrpcClientInitError};
use capwat_types::id::marker::UserMarker;
use capwat_types::Id;

use error_stack::Result;
use tonic::transport::Channel;

#[derive(Debug)]
pub struct DataServiceClient {
  grpc: GrpcClient,
}

impl DataServiceClient {
  #[tracing::instrument]
  pub fn connect(cfg: &GrpcConfig) -> Result<Self, GrpcClientInitError> {
    GrpcClient::new(cfg).map(|grpc| Self { grpc })
  }

  fn establish_grpc_client(&self) -> DataClient<Channel> {
    DataClient::new(self.grpc.get_channel())
  }
}

#[async_trait]
impl drivers::Data for DataServiceClient {
  #[tracing::instrument]
  async fn find_user_by_id(
    &self,
    id: Sensitive<Id<UserMarker>>,
  ) -> KResult<Option<User>> {
    let mut client = self.establish_grpc_client();
    let resp = client
      .find_user_by_id(GrpcRequest::new(proto::FindUserByIdRequest {
        id: id.to_proto()?,
      }))
      .await?
      .into_inner();

    resp.user.map(FromProto::from_proto).transpose()
  }

  #[tracing::instrument]
  async fn find_user_by_login(
    &self,
    email_or_username: Sensitive<&str>,
  ) -> KResult<Option<User>> {
    let mut client = self.establish_grpc_client();
    let resp = client
      .find_user_by_login(GrpcRequest::new(proto::FindUserByLoginRequest {
        email_or_username: email_or_username.into_inner().to_string(),
      }))
      .await?
      .into_inner();

    resp.user.map(FromProto::from_proto).transpose()
  }

  #[tracing::instrument]
  async fn get_secret(&self) -> KResult<Secret> {
    let mut client = self.establish_grpc_client();
    let resp = client.get_secret(GrpcRequest::new(())).await?.into_inner();

    Secret::from_proto(resp)
  }
}
