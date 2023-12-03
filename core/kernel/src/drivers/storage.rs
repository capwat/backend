use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait]
pub trait Storage: Debug + Send + Sync {}
