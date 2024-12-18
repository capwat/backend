use crate::App;
use axum::Router;

mod admin;
mod posts;
mod users;

/// Builds the base router for Capwat API v1.
pub fn build_axum_router(app: App) -> Router {
    Router::new()
        .nest("/admin", self::admin::routes())
        .nest("/posts", self::posts::routes())
        .nest("/users", self::users::routes())
        .with_state(app)
}

/// Converts from raw Capwat schema to schema based on Capwat API v1
mod morphers;
