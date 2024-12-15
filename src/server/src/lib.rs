#![feature(duration_constructors)]
mod extract;

pub mod app;
pub mod auth;
pub mod middleware;
pub mod routes;
pub mod services;
pub mod util;

/// It contains useful utilities for testing the entire server
/// and the implementation of easy `#[capwat_macros::server_test]`
/// when expanded.
#[cfg(test)]
pub(crate) mod test_utils;

pub use self::app::App;

use axum::Router;

/// Builds the entire [Axum router] for establishing a Capwat API server.
pub fn build_axum_router(app: App) -> Router {
    self::middleware::apply(app.clone(), self::routes::build_axum_router(app))
}
