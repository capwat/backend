mod toml_internals;

pub mod db_pools;
pub mod logging;
pub mod server;
pub mod vars;

pub use self::db_pools::{DatabasePool, DatabasePools};
pub use self::logging::Logging;
pub use self::server::Server;

trait ConfigParts {
    type Output;

    /// Merges from another optional value. `self` is the priority.
    fn merge(self, other: Self) -> Self::Output;
}

impl<T> ConfigParts for Option<T> {
    type Output = Option<T>;

    fn merge(self, other: Self) -> Self::Output {
        match (self, other) {
            (Some(this), None) => Some(this),
            (None, Some(other)) => Some(other),
            _ => None,
        }
    }
}
