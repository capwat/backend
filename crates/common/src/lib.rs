#![cfg_attr(test, allow(clippy::unwrap_used))]

pub mod config;
pub mod util;

mod reloader;
pub use reloader::{watch, AppConfig, Config, LoadError};
