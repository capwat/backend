use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use super::category::LoginUserFailed;
use super::{Error, ErrorCategory};

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status_code = match &self.category {
            ErrorCategory::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCategory::ReadonlyMode => StatusCode::SERVICE_UNAVAILABLE,
            ErrorCategory::InvalidRequest => StatusCode::BAD_REQUEST,
            ErrorCategory::Outage => StatusCode::SERVICE_UNAVAILABLE,
            ErrorCategory::InstanceClosed => StatusCode::SERVICE_UNAVAILABLE,
            ErrorCategory::LoginUserFailed(data) => match data {
                LoginUserFailed::InvalidCredientials => StatusCode::FORBIDDEN,
                LoginUserFailed::AccessKeyRequired(..) => StatusCode::BAD_REQUEST,
            },
            ErrorCategory::AccessDenied => StatusCode::UNAUTHORIZED,
            ErrorCategory::KeysExpired => StatusCode::FORBIDDEN,
            ErrorCategory::ExpiredToken => StatusCode::FORBIDDEN,
            // As prescribed from the documentation
            ErrorCategory::NoEmailAddress => StatusCode::FORBIDDEN,
            ErrorCategory::RegisterUserFailed(..) => StatusCode::BAD_REQUEST,
            ErrorCategory::EmailVerificationRequired => StatusCode::FORBIDDEN,
            ErrorCategory::Other(..) => panic!("other error category should not be handled here!"),
        };
        (status_code, Json(self)).into_response()
    }
}
