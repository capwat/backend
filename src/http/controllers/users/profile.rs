use actix_web::{web, HttpResponse};
use serde_json::json;
use thiserror::Error;

use crate::{
  http::{Actor, Error},
  schema::User,
  App,
};

#[tracing::instrument]
pub async fn profile(
  app: web::Data<App>,
  path: web::Path<String>,
  actor: Actor,
) -> Result<HttpResponse, Error> {
  // TODO: Restrict users from signing up using `me` as their username
  let user = if path.as_str() == "me" {
    actor.get_user()?
  } else {
    // TODO: Remove the need of report
    #[derive(Debug, Error)]
    #[error("User not found")]
    struct ResourceError;

    let mut conn = app.db_read_prefer_primary().await?;
    if let Some(user) = User::by_name(&mut *conn, path.as_str()).await? {
      user
    } else {
      return Err(Error::from_context(
        crate::types::Error::NotFound,
        ResourceError,
      ));
    }
  };

  Ok(HttpResponse::Ok().json(json!({
    "id": user.id,
    "created_at": user.created_at,
    "name": user.name,
    "display_name": user.display_name,
  })))
}
