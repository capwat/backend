use std::fmt::Display;

use crate::error::codes;
use whim_derives::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error(code = "codes::SERVER")]
pub enum Error {
  #[error(
    message = "Internal error occurred. Please try again later.",
    subcode = "codes::server::INTERNAL"
  )]
  Internal,
  /// If this error occurs in your client application, you must wait
  /// until the time it takes to probably the server will accept
  /// any write operations.
  ///
  /// ```http
  /// 504 Service Unavailable
  ///
  /// Content-Type: application/json
  /// Retry-After: 2023-01-01T00:00:00Z
  /// ```
  /// ```json
  /// {
  ///   "code": 1,
  ///   "subcode": 2,
  ///   "message": "This server is currently in read-only mode. Please try to do any write operations later."
  /// }
  /// ```
  ///
  /// **Then, the client must go to `/api/v1/status` to get updates
  /// from the server and send request from there if write operations
  /// are accepted again.**
  /// ```json
  /// {
  ///   "database": {
  ///     "can_write": true
  ///   }
  /// }
  /// ```
  ///
  /// **Then, the client MUST send their depending requests to the server.**
  #[error(
    message = "This server is currently in read-only mode. Please try to do any write operations later.",
    subcode = "codes::server::READONLY_MODE"
  )]
  ReadonlyMode,
  #[error(
    message = "This server is experiencing an outage. Please try again later.",
    subcode = "codes::server::OUTAGE"
  )]
  Outage,
}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Internal => f.write_str("Internal error occurred"),
      Error::ReadonlyMode => f.write_str("Tried to write while in read-only mode"),
      Error::Outage => f.write_str("Server experienced outage"),
    }
  }
}
