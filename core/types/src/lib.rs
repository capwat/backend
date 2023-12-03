#![cfg_attr(test, allow(clippy::let_underscore_must_use, clippy::unwrap_used))]
pub(crate) mod internal;

mod sensitive;
#[cfg(feature = "server_impl")]
mod server_impls;

pub mod error;
pub mod forms;
pub mod id;
pub mod timestamp;

pub use error::Error;
pub use id::Id;
pub use sensitive::Sensitive;
pub use timestamp::Timestamp;

// This is to prevent from anyone from performing any database-related
// tests without enabling the `server_impl` feature.
#[cfg(all(test, not(feature = "server_impl"), feature = "db-testing"))]
compile_error!(
  "`server_impl` feature must be enabled to perform database related tests"
);
