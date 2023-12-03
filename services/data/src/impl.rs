use async_trait::async_trait;
use capwat_kernel::drivers::data::types;
use capwat_kernel::drivers::{self, prelude::*};
use capwat_kernel::entity::{Secret, User};
use capwat_types::{id::marker::UserMarker, Id};
use error_stack::{Result, ResultExt};
use std::sync::Arc;
use thiserror::Error;

use crate::config;
use crate::db::ext::SqlxErrorExt;
use crate::db::Database;

#[derive(Debug)]
pub struct DataService {
  config: Arc<config::Database>,
  db: Database,
}

#[derive(Debug, Error)]
#[error("Failed to initialize data service")]
pub struct DataServiceInitError;

impl DataService {
  #[tracing::instrument]
  pub async fn new(
    cfg: config::Database,
  ) -> Result<Self, DataServiceInitError> {
    let db =
      Database::connect(&cfg).await.change_context(DataServiceInitError)?;

    Ok(Self { config: Arc::new(cfg), db })
  }
}

#[async_trait]
impl drivers::Data for DataService {
  #[tracing::instrument]
  async fn create_user(
    &self,
    form: &types::CreateUser<'_>,
  ) -> KResult<Option<User>> {
    let mut conn = self.db.write().await?;

    // TODO: deal with collisions
    let user = sqlx::query_as::<_, User>(
      r"INSERT INTO users (name, email, password_hash) VALUES ($1, $2, $3)",
    )
    .bind(form.name.as_ref())
    .bind(form.email.as_ref().as_deref())
    .bind(form.password_hash.as_ref())
    .fetch_one(&mut *conn)
    .await
    .into_db_error()?;

    Ok(Some(user))
  }

  #[tracing::instrument]
  async fn find_user_by_id(
    &self,
    id: Sensitive<Id<UserMarker>>,
  ) -> KResult<Option<User>> {
    let mut conn = self.db.read_prefer_primary().await?;
    let user = sqlx::query_as::<_, User>(r"SELECT * FROM users WHERE id = $1")
      .bind(id.into_inner())
      .fetch_optional(&mut *conn)
      .await
      .into_db_error()?;

    Ok(user)
  }

  #[tracing::instrument]
  async fn find_user_by_login(
    &self,
    email_or_username: Sensitive<&str>,
  ) -> KResult<Option<User>> {
    let mut conn = self.db.read_prefer_primary().await?;
    let user = sqlx::query_as::<_, User>(
      r"SELECT * FROM users WHERE email = $1 OR username = $1",
    )
    .bind(email_or_username.into_inner())
    .fetch_optional(&mut *conn)
    .await
    .into_db_error()?;

    Ok(user)
  }

  #[tracing::instrument]
  async fn get_secret(&self) -> KResult<Secret> {
    let mut conn = self.db.read().await?;
    let secret = sqlx::query_as::<_, Secret>(r"SELECT * FROM secret")
      .fetch_one(&mut *conn)
      .await
      .into_db_error()?;

    Ok(secret)
  }
}

#[cfg(feature = "grpc")]
use capwat_kernel::drivers::Data;

#[cfg(feature = "grpc")]
#[async_trait]
impl proto::data_server::Data for DataService {
  #[tracing::instrument(skip(request), fields(request = "<hidden>"))]
  async fn find_user_by_login(
    &self,
    request: GrpcRequest<proto::FindUserByLoginRequest>,
  ) -> GrpcResult<GrpcResponse<proto::FindUserReply>> {
    let request = request.into_inner();
    let user =
      Data::find_user_by_login(self, request.email_or_username.as_str().into())
        .await?
        .map(ToProto::to_proto)
        .transpose()?;

    Ok(GrpcResponse::new(proto::FindUserReply { user }))
  }

  #[tracing::instrument(skip(request), fields(request = "<hidden>"))]
  async fn find_user_by_id(
    &self,
    request: GrpcRequest<proto::FindUserByIdRequest>,
  ) -> GrpcResult<GrpcResponse<proto::FindUserReply>> {
    let id = Id::<UserMarker>::from_proto(request.into_inner().id)?;
    let user = Data::find_user_by_id(self, id.into())
      .await?
      .map(ToProto::to_proto)
      .transpose()?;

    Ok(GrpcResponse::new(proto::FindUserReply { user }))
  }

  #[tracing::instrument(skip(_request))]
  async fn get_secret(
    &self,
    _request: GrpcRequest<()>,
  ) -> GrpcResult<GrpcResponse<proto::Secret>> {
    // TODO: authentication
    let secret = Data::get_secret(self).await?;
    Ok(GrpcResponse::new(secret.to_proto()?))
  }
}
