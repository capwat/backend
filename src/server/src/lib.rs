pub mod app;
pub mod auth;
pub mod middleware;
pub mod routes;
pub mod services;

mod extract;
mod util;

pub use self::app::App;

use axum::Router;

/// Builds the entire [Axum router] for establishing a Capwat API server.
#[must_use]
pub fn build_axum_router(app: App) -> Router {
    self::middleware::apply(app.clone(), self::routes::build_axum_router(app))
}
