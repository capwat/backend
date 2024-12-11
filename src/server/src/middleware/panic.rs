use axum::response::{IntoResponse, Response};
use capwat_error::Error;
use std::any::Any;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
#[error("Route controller got panicked")]
struct Panicked;

#[tracing::instrument(skip_all, name = "middleware.catch_panic")]
pub fn catch_panic(err: Box<dyn Any + Send + 'static>) -> Response {
    let data = if let Some(s) = err.downcast_ref::<String>() {
        s.to_string()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "<unknown>".into()
    };

    Error::unknown_generic(Panicked)
        .attach_printable(format!("message: {data}"))
        .into_api_error()
        .into_response()
}
