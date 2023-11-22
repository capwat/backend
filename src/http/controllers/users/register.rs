use actix_web::{
  web::{self, Json},
  HttpResponse,
};
use futures::StreamExt;
use sha2::Digest;
use sqlx::FromRow;
use validator::{Validate, ValidateError};

use crate::{
  database::error::ErrorExt, http::Error, schema::User, types::form::users::register, App,
};

#[tracing::instrument]
pub async fn register(
  app: web::Data<App>,
  form: Json<register::Request>,
) -> Result<HttpResponse, Error> {
  form.validate()?;

  #[derive(Debug, FromRow)]
  struct Query {
    name_exists: bool,
    email_exists: bool,
  }

  let mut conn = app.db_write().await?;
  let mut query = sqlx::QueryBuilder::new("select (name = $1) as name_exists");

  if form.email.is_some() {
    query.push(", (email = $2) as email_exists");
  }
  query.push(" from users where name = $1");
  if form.email.is_some() {
    query.push(" or email = $2");
  }

  let mut query = query.build_query_as::<Query>().bind(form.username.as_str());
  if let Some(email) = form.email.as_deref() {
    query = query.bind(email);
  }

  let mut count = 0;
  let mut email_exists = false;
  let mut username_exists = false;

  let mut stream = query.fetch(&mut *conn);
  while let Some(row) = stream.next().await {
    if count == 2 {
      // this is to avoid DDOS
      break;
    }

    let row = row.into_db_error()?;
    email_exists = email_exists || row.email_exists;
    username_exists = username_exists || row.name_exists;
    count += 1;
  }
  drop(stream);

  if email_exists || username_exists {
    let mut err = ValidateError::field_builder();
    if email_exists {
      let mut msg = ValidateError::msg_builder();
      msg.insert("This email address exists");
      err.insert("email", msg.build());
    }
    if username_exists {
      let mut msg = ValidateError::msg_builder();
      msg.insert("This username exists");
      err.insert("username", msg.build());
    }
    return Err(err.build().into());
  }

  // TODO: Not secure right now but we need to get this asap
  let mut hasher = sha2::Sha512::default();
  hasher.update(format!(
    "{}:{}",
    form.username.as_str(),
    form.password.as_str()
  ));

  let password_hash = hex::encode(hasher.finalize());

  // Attempting to insert user right now!
  let _new_user = sqlx::query_as::<_, User>(
    r#"INSERT INTO "users" (name, email, password_hash)
       VALUES ($1, $2, $3)
       RETURNING *"#,
  )
  .bind(form.username.as_str())
  .bind(form.email.as_deref())
  .bind(password_hash)
  .fetch_one(&mut *conn)
  .await
  .into_db_error()?;

  Ok(HttpResponse::Created().json(register::Response {
    verification_required: form.email.is_some(),
  }))
}
