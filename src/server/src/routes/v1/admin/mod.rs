use crate::App;
use axum::routing::get;
use axum::Router;

pub mod local_instance;

pub fn routes() -> Router<App> {
    Router::new().route(
        "/instance/settings",
        get(self::local_instance::get_settings),
    )
}
