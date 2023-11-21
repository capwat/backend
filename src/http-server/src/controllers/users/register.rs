use actix_web::{
  web::{self, Json},
  HttpResponse,
};
use whim_types::form::users::register;

use crate::{error::Error, App};

#[tracing::instrument]
pub async fn register(
  app: web::Data<App>,
  form: Json<register::Request>,
) -> Result<HttpResponse, Error> {
  app.db_read().await?;
  Ok(HttpResponse::Created().body("Hi"))
}
