use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed to commit transaction")]
pub struct CommitFailed;

#[derive(Debug, Error)]
#[error("Failed to rollback transaction")]
pub struct RollbackFailed;

#[derive(Debug, Error)]
#[error("Failed to start transaction")]
pub struct BeginFailed;
