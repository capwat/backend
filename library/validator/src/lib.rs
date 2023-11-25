#![cfg_attr(test, allow(clippy::unwrap_used))]

mod error;
mod std_impl;

pub use error::*;
pub mod extras;

pub trait Validate {
  fn validate(&self) -> Result<(), ValidateError>;
}

pub trait HasLength {
  fn length(&self) -> usize;
}

pub use validator_derive::Validate;
