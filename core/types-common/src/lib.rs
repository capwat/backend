pub(crate) mod internal;

pub mod error;
pub mod id;
pub mod timestamp;

pub use timestamp::Timestamp;

#[cfg(feature = "full")]
mod sensitive;
#[cfg(feature = "full")]
pub use sensitive::Sensitive;

/// Keeps the raw sensitive data in memory but it cannot be
/// accidentally leaked through the console or logs.
///
/// If `server` feature is disabled, this type is that directly
/// referred to the generic argument.
#[cfg(not(feature = "full"))]
pub type Sensitive<T> = T;
