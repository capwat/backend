use capwat_kernel::{
  config,
  db::{Database, SqlxErrorExt},
  entity::{
    id::{marker::UserMarker, Id},
    User,
  },
  services::{impl_dev::*, DataService},
};
use error_stack::{Result as StackResult, ResultExt};
use thiserror::Error;
use tonic::{Request, Response, Status};

use crate::protobuf::{
  self, data_server::Data, GetUserByIdRequest, GetUserByLoginRequest,
  GetUserResponse,
};

#[derive(Debug)]
pub struct ServerLayer {
  db: Database,
}

#[derive(Debug, Error)]
#[error("Failed to connect to the database")]
pub struct ServerLayerError;

impl ServerLayer {
  #[tracing::instrument]
  pub async fn connect(
    db: &config::Database,
  ) -> StackResult<Self, ServerLayerError> {
    let db = Database::connect(db).await.change_context(ServerLayerError)?;

    Ok(Self { db })
  }
}

#[async_trait]
impl DataService for ServerLayer {
  #[tracing::instrument]
  async fn find_user_by_id(
    &self,
    id: Id<UserMarker>,
  ) -> ServiceResult<Option<User>> {
    let mut conn = self.db.read_prefer_primary().await?;
    let user = sqlx::query_as::<_, User>(r"SELECT * FROM users WHERE id = $1")
      .bind(id)
      .fetch_optional(&mut *conn)
      .await
      .into_db_error()?;

    Ok(user)
  }

  #[tracing::instrument]
  async fn find_user_by_login(
    &self,
    email_or_username: &str,
  ) -> ServiceResult<Option<User>> {
    let mut conn = self.db.read_prefer_primary().await?;
    let user = sqlx::query_as::<_, User>(
      r"SELECT * FROM users WHERE email = $1 OR username = $1",
    )
    .bind(email_or_username)
    .fetch_optional(&mut *conn)
    .await
    .into_db_error()?;

    Ok(user)
  }
}

#[async_trait]
impl Data for ServerLayer {
  async fn get_user_by_login(
    &self,
    request: Request<GetUserByLoginRequest>,
  ) -> Result<Response<GetUserResponse>, Status> {
    let request = request.into_inner();
    let user =
      <_ as DataService>::find_user_by_login(self, &request.email_or_username)
        .await
        .map_err(|e| e.into_tonic_status())?;

    let user = user.map(|v| protobuf::User {
      id: v.id.get(),
      created_at: v.created_at.to_string(),
      name: v.name,
      email: v.email,
      display_name: v.display_name,
      password_hash: v.password_hash,
      updated_at: v.updated_at.map(|v| v.to_string()),
    });

    Ok(Response::new(GetUserResponse { user }))
  }

  async fn get_user_by_id(
    &self,
    request: Request<GetUserByIdRequest>,
  ) -> Result<Response<GetUserResponse>, Status> {
    let request = request.into_inner();

    // This is served as an invalid request
    if let Some(id) = Id::<UserMarker>::new_checked(request.id) {
      let user = <_ as DataService>::find_user_by_id(self, id)
        .await
        .map_err(|e| e.into_tonic_status())?;

      let user = user.map(|v| protobuf::User {
        id: v.id.get(),
        created_at: v.created_at.to_string(),
        name: v.name,
        email: v.email,
        display_name: v.display_name,
        password_hash: v.password_hash,
        updated_at: v.updated_at.map(|v| v.to_string()),
      });

      Ok(Response::new(GetUserResponse { user }))
    } else {
      Ok(Response::new(GetUserResponse { user: None }))
    }
  }
}
