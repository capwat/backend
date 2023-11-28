use async_trait::async_trait;
use capwat_kernel::{
  db::Database,
  entity::{
    id::{marker::UserMarker, Id},
    User,
  },
  error::{ErrorStackContext, Result, StdContext},
  services::{self, DataError},
};

pub struct Layer {
  db: Database,
}

#[async_trait]
impl services::Data for Layer {
  async fn find_user_by_id(
    &self,
    id: Id<UserMarker>,
  ) -> Result<Option<User>, DataError> {
    let mut conn = self.db.read_prefer_primary().await.into_capwat_error()?;
    sqlx::query_as::<_, User>(r#"SELECT * FROM "users" WHERE id = $1"#)
      .bind(id)
      .fetch_optional(&mut *conn)
      .await
      .into_capwat_error()
  }
}
