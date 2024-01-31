pub(crate) mod internal;

pub mod error;
pub mod form;
pub mod id;
pub mod timestamp;

pub use timestamp::Timestamp;

#[cfg(feature = "server")]
mod sensitive;
#[cfg(feature = "server")]
pub use sensitive::Sensitive;

/// Keeps the raw sensitive data in memory but it cannot be
/// accidentally leaked through the console or logs.
///
/// If `server` feature is disabled, this type is that directly
/// referred to the generic argument.
#[cfg(not(feature = "server"))]
pub type Sensitive<T> = T;
