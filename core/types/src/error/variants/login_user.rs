use serde::{Deserialize, Serialize};

use crate::Timestamp;

#[derive(Debug, PartialEq, Eq)]
pub enum LoginUser {
  InvalidCredientials,
  Banned(LoginUserBanData),
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct LoginUserBanData {
  pub appealable: bool,
  pub banned_until: Option<Timestamp>,
  pub reason: String,
  pub violations: Vec<String>,
}
