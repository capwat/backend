use crate::{
  types::id::{marker::UserMarker, Id},
  util::Sensitive,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Request {
  #[validate(length(min = 1, max = 128))]
  pub username_or_email: Sensitive<String>,
  #[validate(length(min = 12, max = 128))]
  pub password: Sensitive<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
  pub id: Id<UserMarker>,
  pub token: Sensitive<String>,
}
