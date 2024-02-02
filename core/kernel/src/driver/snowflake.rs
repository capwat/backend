use async_trait::async_trait;
use capwat_types_common::id::{AnyMarker, Id};
use std::fmt::Debug;

use crate::Result;

#[async_trait]
pub trait Snowflake: Debug + Send + Sync {
    /// Generates a new unique snowflake ID.
    async fn next_id(&self) -> Result<Id<AnyMarker>>;
}
