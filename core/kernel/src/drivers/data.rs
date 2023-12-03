use async_trait::async_trait;
use capwat_types::id::{marker::UserMarker, Id};
use capwat_types::Sensitive;
use std::fmt::Debug;

use self::types::CreateUser;

use super::prelude::*;
use crate::entity::{Secret, User};

#[async_trait]
pub trait Data: Debug + Send + Sync {
  async fn create_user(&self, form: &CreateUser<'_>) -> KResult<Option<User>>;

  async fn find_user_by_id(
    &self,
    id: Sensitive<Id<UserMarker>>,
  ) -> KResult<Option<User>>;

  async fn find_user_by_login(
    &self,
    email_or_username: Sensitive<&str>,
  ) -> KResult<Option<User>>;

  // transporting secrets is absolutely a BIG RISK
  // without sort of tls encryption
  async fn get_secret(&self) -> KResult<Secret>;
}

pub mod types {
  use capwat_types::Sensitive;

  #[derive(Debug)]
  pub struct CreateUser<'a> {
    pub name: Sensitive<&'a str>,
    pub email: Sensitive<Option<&'a str>>,
    pub password_hash: Sensitive<&'a str>,
  }
}

#[cfg(test)]
pub mod tests {
  use async_trait::async_trait;
  use capwat_types::id::marker::UserMarker;
  use capwat_types::{Id, Timestamp};
  use std::sync::Arc;
  use tokio::sync::RwLock;

  use crate::drivers::prelude::*;
  use crate::entity::{Secret, User};

  #[derive(Debug)]
  pub struct MockDataService {
    secret: Secret,
    users: Arc<RwLock<Vec<User>>>,
  }

  #[allow(clippy::new_without_default)]
  impl MockDataService {
    #[must_use]
    pub fn new() -> Self {
      Self {
        secret: Secret {
          id: Id::new(1),
          jwt: "Hello-World".to_string().into(),
        },
        users: Arc::default(),
      }
    }
  }

  #[async_trait]
  impl super::Data for MockDataService {
    async fn create_user(
      &self,
      form: &super::types::CreateUser<'_>,
    ) -> KResult<Option<User>> {
      let mut users = self.users.write().await;
      let exists = users.iter().any(|v| {
        let email = v
          .email
          .as_ref()
          .zip(form.email.as_ref().as_ref())
          .map(|(a, b)| a == b)
          .unwrap_or_default();

        v.name.eq(form.name.as_ref()) && email
      });

      if exists {
        Ok(None)
      } else {
        let users_len = users.len();
        let user = User {
          id: Id::new(users_len as u64),
          created_at: Timestamp::now().into(),
          name: form.name.into_inner().to_string(),
          email: form.email.as_ref().map(std::string::ToString::to_string),
          display_name: None,
          password_hash: form.password_hash.into_inner().to_string(),
          updated_at: None,
        };
        users.push(user.clone());
        Ok(Some(user))
      }
    }

    async fn find_user_by_id(
      &self,
      id: Sensitive<Id<UserMarker>>,
    ) -> KResult<Option<User>> {
      let id = id.into_inner();
      Ok(self.users.read().await.iter().find(|v| v.id == id).cloned())
    }

    async fn find_user_by_login(
      &self,
      email_or_username: Sensitive<&str>,
    ) -> KResult<Option<User>> {
      let email_or_username = email_or_username.into_inner();
      let user = self
        .users
        .read()
        .await
        .iter()
        .find(|user| {
          let matched_email = user
            .email
            .as_ref()
            .map(|v| v == email_or_username)
            .unwrap_or_default();
          let matched_username = user.name == email_or_username;
          matched_email || matched_username
        })
        .cloned();

      Ok(user)
    }

    // transporting secrets is absolutely a BIG RISK
    // without sort of tls encryption
    async fn get_secret(&self) -> KResult<Secret> {
      Ok(self.secret.clone())
    }
  }
}
