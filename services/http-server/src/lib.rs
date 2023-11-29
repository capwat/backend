use std::fmt::Debug;

use async_trait::async_trait;
use capwat_kernel::error::Result;
use serde::Serialize;

mod forms;

pub mod app;
pub use app::App;

#[async_trait]
pub trait Perform: Debug + Send + Sync + validator::Validate {
  type Response: Serialize;

  async fn perform(&self, app: App) -> Result<Self::Response>;
}
