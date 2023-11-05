use thiserror::Error;

mod database;
mod server;

pub use database::{Database, DbPoolConfig};
pub use server::Server;

#[derive(Debug, Error)]
#[error("Failed to load configuration")]
pub struct ParseError;
