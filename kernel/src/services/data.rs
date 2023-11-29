use async_trait::async_trait;
use capwat_types::id::{marker::UserMarker, Id};
use std::fmt::Debug;

use crate::{entity::User, error::Result};

#[async_trait]
pub trait DataService: Debug + Send + Sync {
  async fn find_user_by_id(&self, id: Id<UserMarker>) -> Result<Option<User>>;
  async fn find_user_by_login(
    &self,
    email_or_username: &str,
  ) -> Result<Option<User>>;
}
