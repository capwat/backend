#![feature(allocator_api, closure_track_caller, trait_upcasting)]
mod error;
mod fmt;
mod internal;

pub mod context;
pub mod ext;
pub mod middleware;

pub use self::error::Error;
pub use capwat_api_types::{Error as ApiError, ErrorCategory as ApiErrorCategory};

pub type Result<T, C = internal::NoContext> = std::result::Result<T, Error<C>>;
