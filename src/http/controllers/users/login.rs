use actix_web::{
  web::{self, Json},
  HttpResponse,
};
use sha2::Digest;
use validator::{Validate, ValidateError};

use crate::{
  http::{Error, Jwt},
  schema::User,
  types::form::users::login,
  App,
};

#[tracing::instrument]
pub async fn login(app: web::Data<App>, form: Json<login::Request>) -> Result<HttpResponse, Error> {
  form.validate()?;

  // We need to get the latest info as soon as possible
  let mut conn = app.db_read_prefer_primary().await?;

  let Some(user) = User::by_name_or_email(&mut conn, &form.username_or_email).await? else {
    let mut error = ValidateError::field_builder();
    let mut contents = ValidateError::msg_builder();
    contents.insert("Invalid credientials");
    error.insert("username_or_email", contents.build());
    return Err(error.build().into());
  };

  drop(conn);

  // TODO: Not secure right now but we need to get this asap
  let mut hasher = sha2::Sha512::default();
  hasher.update(format!("{}:{}", user.name.as_str(), form.password.as_str()));

  let attempt_password_hash = hex::encode(hasher.finalize());

  let mut matched = true;
  for (a, b) in user
    .password_hash
    .chars()
    .zip(attempt_password_hash.chars())
  {
    matched = matched && (a == b);
  }

  if !matched {
    let mut error = ValidateError::field_builder();
    let mut contents = ValidateError::msg_builder();
    contents.insert("Invalid credientials");
    error.insert("username_or_email", contents.build());
    Err(error.build().into())
  } else {
    let jwt = Jwt::encode(user.id, app.clone()).await;
    Ok(HttpResponse::Ok().json(login::Response {
      id: user.id,
      token: jwt.into(),
    }))
  }
}
