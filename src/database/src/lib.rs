mod pool;

pub mod error;
pub use error::{Error, Result};
pub use pool::Pool;

pub type Transaction<'a> = sqlx::Transaction<'a, sqlx::Postgres>;
pub type PoolConnection = sqlx::pool::PoolConnection<sqlx::Postgres>;
pub type Connection = sqlx::PgConnection;
