use axum::extract::Query;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use capwat_api_types::routes::posts::GetPostFeed;
use capwat_error::ApiError;

use super::build_api_post_from_view;
use crate::extract::Json;
use crate::extract::SessionUser;
use crate::services;
use crate::App;

pub async fn get_post_feed(
    app: App,
    session_user: SessionUser,
    Query(data): Query<GetPostFeed>,
) -> Result<Response, ApiError> {
    let request = services::posts::GetPostFeed {
        page: data.pagination.page,
        limit: data.pagination.limit,
    };

    let response = request
        .perform(&app, &session_user)
        .await?
        .into_iter()
        .map(|view| build_api_post_from_view(view))
        .collect::<Vec<_>>();

    Ok(Json(response).into_response())
}

pub fn routes() -> Router<App> {
    Router::new().route("/feed", get(get_post_feed))
}
