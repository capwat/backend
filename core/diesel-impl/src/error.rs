use capwat_kernel::error::ext::IntoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PoolError {
    #[error("Received a pool error")]
    General,
    #[error("Unhealthy database pool")]
    UnhealthyPool,
}

impl IntoError for PoolError {
    fn into_error(self) -> capwat_kernel::Error {
        capwat_kernel::Error::internal(self)
    }
}

#[derive(Debug, Error)]
#[error("Failed to run migrations")]
pub struct MigrationError;
