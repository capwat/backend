use async_trait::async_trait;
use capwat_types::{
  error::{codes, ErrorCategory},
  id::{marker::UserMarker, Id},
};
use thiserror::Error;

use crate::{entity::User, error::Result};

#[async_trait]
pub trait Data {
  async fn find_user_by_id(
    &self,
    id: Id<UserMarker>,
  ) -> Result<Option<User>, DataError>;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DataError {}

impl ErrorCategory for DataError {
  fn code() -> u32 {
    codes::users::CATEGORY
  }

  fn subcode(&self) -> Option<u32> {
    None
  }

  fn server_message(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    todo!()
  }

  fn message(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    todo!()
  }

  fn needs_data_serialization(&self) -> bool {
    todo!()
  }

  fn deserialize_data<'de, D>(
    subcode: Option<u32>,
    deserializer: D,
  ) -> std::result::Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
    Self: Sized,
  {
    todo!()
  }
}
