use thiserror::Error;

mod auth;
mod database;
mod server;

pub use auth::Auth;
pub use database::{Database, DbPoolConfig};
pub use server::Server;

#[derive(Debug, Error)]
#[error("Failed to load configuration")]
pub struct LoadError;
