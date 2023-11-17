use chrono::NaiveDateTime;
use sqlx::FromRow;
use whim_database::{error::ErrorExt, Connection, Result};

use crate::id::UserId;

#[derive(Debug, FromRow, PartialEq, Eq)]
pub struct User {
  pub id: UserId,
  pub created_at: NaiveDateTime,
  pub name: String,
  pub display_name: Option<String>,
  pub email: Option<String>,
  pub password_hash: String,
  pub updated_at: Option<NaiveDateTime>,
}

impl User {
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
