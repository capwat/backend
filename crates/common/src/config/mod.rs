// TODO: Implement config hot reloading
use thiserror::Error;

mod auth;
mod database;

pub use auth::Auth;
pub use database::{Database, DatabasePool};

#[derive(Debug, Error)]
#[error("Failed to load configuration")]
pub struct LoadError;
