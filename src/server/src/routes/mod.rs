use crate::App;

use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Router;
use capwat_error::{ApiError, ApiErrorCategory};

mod v1;

/// Builds an [axum router] based all routers available for the Capwat API.
///
/// [axum router]: axum::Router
pub fn build_axum_router(app: App) -> Router {
    Router::new()
        .nest("/api/v1", self::v1::build_axum_router(app.clone()))
        .nest("/api/", self::v1::build_axum_router(app))
        .method_not_allowed_fallback(method_not_allowed_route)
        .fallback(not_found_route)
}

async fn method_not_allowed_route() -> Response {
    ApiError::new(ApiErrorCategory::InvalidRequest).into_response()
}

async fn not_found_route(method: Method) -> Response {
    match method {
        Method::HEAD => StatusCode::NOT_FOUND.into_response(),
        _ => ApiError::new(ApiErrorCategory::NotFound).into_response(),
    }
}
