use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
  id::{marker::UserMarker, Id},
  Sensitive,
};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Login {
  #[validate(length(min = 1, max = 128))]
  pub username_or_email: Sensitive<String>,
  #[validate(length(min = 12, max = 128))]
  pub password: Sensitive<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct LoginResponse {
  pub id: Id<UserMarker>,
  pub token: Sensitive<String>,
}
