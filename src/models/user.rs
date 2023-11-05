use chrono::NaiveDateTime;
use sqlx::FromRow;

use super::id::UserId;

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
