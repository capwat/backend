use serde::{Deserialize, Serialize};

use crate::Sensitive;

#[derive(Debug, Deserialize, Serialize)]
pub struct Login {
  pub username_or_email: Sensitive<String>,
  pub password: Sensitive<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Register {
  pub username: Sensitive<String>,
  pub email: Sensitive<Option<String>>,
  pub password: Sensitive<String>,
  pub confirm_password: Sensitive<String>,
}
