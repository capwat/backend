use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Unknown {
  pub code: u32,
  pub subcode: Option<u32>,
  pub message: String,
  pub data: Option<serde_value::Value>,
}

impl Display for Unknown {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.message.fmt(f)
  }
}
