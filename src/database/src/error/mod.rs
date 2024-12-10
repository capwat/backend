use thiserror::Error;

#[derive(Debug, Error)]
#[error("Could not build database pool")]
pub struct ConnectError;

#[derive(Debug, Error)]
pub enum AcquireError {
    #[error("Could not acquire database connection")]
    General,
    #[error("Pool is unhealthy")]
    Unhealthy,
}

#[derive(Debug, Error)]
#[error("Failed to perform database migrations")]
pub struct MigrationError;

#[derive(Debug, Error)]
#[error("Failed to commit transaction")]
pub struct CommitTransactError;

#[derive(Debug, Error)]
#[error("Failed to rollback transaction")]
pub struct RollbackTransactError;

#[derive(Debug, Error)]
#[error("Failed to start transaction")]
pub struct BeginTransactError;
