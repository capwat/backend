use axum::routing::{get, post};
use axum::Router;

use crate::App;

mod admin;
mod users;

/// Builds the base router for Capwat API v1.
pub fn build_axum_router(app: App) -> Router {
    Router::new()
        .route(
            "/admin/instance/settings",
            get(self::admin::local_instance::get_settings),
        )
        .route("/users/:id/follow", post(self::users::profile::follow))
        .route("/users/@me", get(self::users::profile::local::view))
        .route(
            "/users/@me/posts",
            post(self::users::profile::local::publish_post),
        )
        .route("/users/login", post(self::users::login))
        .route("/users/register", post(self::users::register))
        .with_state(app)
}
