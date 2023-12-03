use capwat_types::id::{marker::SecretMarker, Id};
use capwat_types::Sensitive;
use sqlx::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Secret {
  pub id: Id<SecretMarker>,
  pub jwt: Sensitive<String>,
}
