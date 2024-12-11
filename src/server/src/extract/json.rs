use axum::extract::Request;
use axum::extract::{rejection::JsonRejection as AxumError, FromRequest};
use axum::http::{header, HeaderValue};
use axum::response::{IntoResponse, Response};
use bytes::{BufMut, BytesMut};
use capwat_error::{ApiError, ApiErrorCategory, Error};
use thiserror::Error;
use tracing::warn;

/// Local version of [`axum::Json`] but it makes an HTTP response based
/// on Capwat API's error schema if it fails to deserialize an object.
pub struct Json<T>(pub T);

#[derive(Debug, Error)]
#[error("Could not serialize response to JSON body")]
struct JsonSerializationError;

impl<T> IntoResponse for Json<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> Response {
        // Use a small initial capacity of 128 bytes like serde_json::to_vec
        // https://docs.rs/serde_json/1.0.82/src/serde_json/ser.rs.html#2189
        let mut buf = BytesMut::with_capacity(128).writer();
        match serde_json::to_writer(&mut buf, &self.0) {
            Ok(()) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                )],
                buf.into_inner().freeze(),
            )
                .into_response(),
            Err(error) => Error::unknown_generic(error)
                .change_context_slient(JsonSerializationError)
                .into_api_error()
                .into_response(),
        }
    }
}

#[axum::async_trait]
impl<T, S> FromRequest<S> for Json<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(inner) => Ok(Json(inner.0)),
            Err(error) => Err(ApiError::new(ApiErrorCategory::InvalidRequest)
                .message(match error {
                    AxumError::JsonDataError(json_data_error) => json_data_error.body_text(),
                    AxumError::JsonSyntaxError(json_syntax_error) => json_syntax_error.body_text(),
                    AxumError::MissingJsonContentType(..) => "Invalid content type".to_string(),
                    AxumError::BytesRejection(bytes_rejection) => bytes_rejection.body_text(),
                    inner => {
                        warn!("unhandled axum::JsonRejection category: {inner:?}");
                        return Err(Error::unknown_generic(inner)
                            .into_api_error()
                            .into_response());
                    }
                })
                .into_response()),
        }
    }
}
