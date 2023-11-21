use chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::{
  database::{error::ErrorExt, Connection, Result},
  types::id::{marker::UserMarker, Id},
};

#[derive(Debug, FromRow, PartialEq, Eq)]
pub struct User {
  pub id: Id<UserMarker>,
  pub created_at: NaiveDateTime,
  pub name: String,
  pub display_name: Option<String>,
  pub email: Option<String>,
  pub password_hash: String,
  pub updated_at: Option<NaiveDateTime>,
}

impl User {
  #[tracing::instrument(skip(id), fields(id = "<hidden>"))]
  pub async fn by_id(conn: &mut Connection, id: Id<UserMarker>) -> Result<Option<Self>> {
    sqlx::query_as::<_, Self>(r#"SELECT * FROM "users" WHERE id = $1"#)
      .bind(id)
      .fetch_optional(conn)
      .await
      .into_db_error()
  }

  // Its function name is ridiculously long tbh
  #[tracing::instrument(skip(condition), fields(condition = "<hidden>"))]
  pub async fn by_name_or_email(conn: &mut Connection, condition: &str) -> Result<Option<Self>> {
    sqlx::query_as::<_, Self>(r#"SELECT * FROM "users" WHERE name = $1 OR email = $1"#)
      .bind(condition)
      .fetch_optional(conn)
      .await
      .into_db_error()
  }

  #[tracing::instrument(skip(condition), fields(condition = "<hidden>"))]
  pub async fn by_email(conn: &mut Connection, condition: &str) -> Result<Option<Self>> {
    sqlx::query_as::<_, Self>(r#"SELECT * FROM "users" WHERE email = $1"#)
      .bind(condition)
      .fetch_optional(conn)
      .await
      .into_db_error()
  }

  #[tracing::instrument(skip(condition), fields(condition = "<hidden>"))]
  pub async fn by_name(conn: &mut Connection, condition: &str) -> Result<Option<Self>> {
    sqlx::query_as::<_, Self>(r#"SELECT * FROM "users" WHERE name = $1"#)
      .bind(condition)
      .fetch_optional(conn)
      .await
      .into_db_error()
  }
}
