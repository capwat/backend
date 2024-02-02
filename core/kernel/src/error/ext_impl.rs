// Rust does not allow us to implement traits outside of their
// crate with objects from different crates.
//
// This module solves the issue at the expense of making this crate
// free from any vendor/dependency lock-in.
use super::{Category, Error};
use crate::util::io::MutWriter;

use actix_web::body::BoxBody;
use actix_web::http::header::{self, TryIntoHeaderValue};
use actix_web::http::StatusCode;
use actix_web::web::BytesMut;
use actix_web::{HttpResponse, ResponseError};
use capwat_types_common::error::ErrorCode;

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        // future proof way of determining error codes
        let category = self.as_category();
        let code = ErrorCode::from_code(category.code());
        let subcode = category.subcode();

        match code {
            ErrorCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorCode::ReadonlyMode => StatusCode::SERVICE_UNAVAILABLE,
            ErrorCode::NotAuthenticated => StatusCode::UNAUTHORIZED,
            ErrorCode::NotFound => StatusCode::NOT_FOUND,
            ErrorCode::LoginUser => {
                use capwat_types_common::error::LoginUserCode;
                let subcode = subcode.map(LoginUserCode::from_subcode);
                match subcode {
                    Some(LoginUserCode::Banned) => StatusCode::FORBIDDEN,
                    Some(LoginUserCode::InvalidCredientials) => {
                        StatusCode::BAD_REQUEST
                    },
                    Some(LoginUserCode::Unknown(..)) | None => {
                        StatusCode::BAD_REQUEST
                    },
                }
            },
            ErrorCode::Unknown(..) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(
        &self,
    ) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut res = HttpResponse::new(self.status_code());
        let mut buf = BytesMut::new();

        let mut writer = MutWriter(&mut buf);
        if let Err(error) = serde_json::to_writer(&mut writer, &self.category) {
            tracing::error!(?error, "Failed to serialize error");
            writer.0.clear();

            serde_json::to_writer(&mut writer, &self.category)
                .expect("Cannot serialize internal error");
        }

        let mime = mime::APPLICATION_JSON.try_into_value().expect(
            "Failed to convert `application/json` mime to header value",
        );
        res.headers_mut().insert(header::CONTENT_TYPE, mime);
        res.set_body(BoxBody::new(buf))
    }
}

impl super::ext::IntoError for diesel::ConnectionError {
    fn into_error(self) -> Error {
        match self {
            diesel::ConnectionError::CouldntSetupConfiguration(n) => {
                n.into_error()
            },
            _ => Error::internal(self),
        }
    }
}

impl super::ext::IntoError for diesel::result::Error {
    fn into_error(self) -> Error {
        match self {
            diesel::result::Error::DatabaseError(_, ref info)
                if info.message().ends_with("read-only transaction") =>
            {
                Error::from_context(Category::ReadonlyMode, self)
            },
            diesel::result::Error::NotFound => {
                Error::from_context(Category::NotFound, self)
            },
            _ => Error::internal(self),
        }
    }
}
