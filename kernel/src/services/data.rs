use async_trait::async_trait;
use capwat_types::id::{marker::UserMarker, Id};

use crate::{entity::User, error::Result};

#[async_trait]
pub trait DataService {
  async fn find_user_by_id(&self, id: Id<UserMarker>) -> Result<Option<User>>;
}
