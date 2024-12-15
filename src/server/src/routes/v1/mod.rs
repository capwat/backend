use crate::App;
use axum::Router;

mod admin;
mod users;

/// Builds the base router for Capwat API v1.
pub fn build_axum_router(app: App) -> Router {
    Router::new()
        .nest("/admin", self::admin::routes())
        .nest("/users", self::users::routes())
        .with_state(app)
}
