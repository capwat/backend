use super::AppErrorType;
use serde_json::json;
use thiserror::Error;

impl AppErrorType for validator::ValidateError {
    fn json_metadata(&self) -> serde_json::Result<serde_json::Value> {
        Ok(json!({
            "error": "invalid_data",
            "message": "Your request contains invalid data",
            "data": serde_json::to_value(self)?
        }))
    }
}

fn internal_err() -> serde_json::Value {
    json!({
      "error": "internal",
      "message": INTERNAL_ERROR,
    })
}

// Internal error occurred.
#[derive(Debug, Error)]
#[error("Something went wrong when performing an action.")]
pub struct InternalError;

const INTERNAL_ERROR: &str = "Internal error occurred, please try again later.";

impl AppErrorType for InternalError {
    fn json_metadata(&self) -> serde_json::Result<serde_json::Value> {
        Ok(internal_err())
    }
}
