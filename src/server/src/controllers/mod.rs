use crate::App;
use axum::routing::{get, post};
use axum::Router;

mod users;

pub async fn index() -> &'static str {
    "Hello, World!"
}

/// Builds the base router for Capwat HTTP API v1.
fn build_v1_router(app: App) -> Router {
    Router::new()
        .route("/", get(index))
        // .route("/users/login", post(users::login::login))
        .route("/users/register", post(users::register::register))
        .with_state(app)
}

/// Builds an [axum router] based all controllers available for
/// the Capwat HTTP API.
///
/// [axum router]: axum::Router
pub fn build_axum_router(app: App) -> Router {
    Router::new()
        .nest("/v1", build_v1_router(app.clone()))
        .nest("/", build_v1_router(app))
}
