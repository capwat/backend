use capwat_types::id::{marker::UserMarker, Id};
use chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct User {
  pub id: Id<UserMarker>,
  pub created_at: NaiveDateTime,
  pub name: String,
  pub email: Option<String>,
  pub display_name: Option<String>,
  pub password_hash: String,
  pub updated_at: Option<NaiveDateTime>,
}
