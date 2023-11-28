mod data;
pub use data::DataService;

pub mod impl_dev {
  pub use crate::error::{
    ErrorStackContext, Result as ServiceResult, StdContext,
  };
  pub use async_trait::async_trait;
}
