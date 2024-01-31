use async_trait::async_trait;
use capwat_types::{
    id::{Id, UserMarker},
    Sensitive,
};

use crate::entity::User;
use crate::Result;

#[async_trait]
pub trait Service: Send + Sync {
    async fn create(&self, input: CreateUser<'_>) -> Result<User>;
    async fn find_by_id(&self, id: Id<UserMarker>) -> Result<Option<User>>;
    async fn find_by_name(&self, name: &str) -> Result<Option<User>>;
}

#[derive(Debug)]
pub struct CreateUser<'a> {
    pub name: Sensitive<&'a str>,
    pub email: Sensitive<Option<&'a str>>,
    pub password_hash: Sensitive<&'a str>,
}
