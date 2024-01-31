use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed to start transaction")]
pub struct BeginFailed;

#[derive(Debug, Error)]
#[error("Already committed transaction")]
pub struct AlreadyCommitted;

#[derive(Debug, Error)]
#[error("Already rollbacked transaction")]
pub struct AlreadyRollbacked;

#[derive(Debug, Error)]
#[error("Failed to commit transaction")]
pub struct CommitFailed;

#[derive(Debug, Error)]
#[error("Failed to rollback transaction")]
pub struct RollbackFailed;
