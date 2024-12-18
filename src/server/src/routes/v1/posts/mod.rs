use axum::extract::Query;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use capwat_api_types::routes::posts::ListPostRecommendations;
use capwat_error::ApiError;

use crate::extract::Json;
use crate::extract::SessionUser;
use crate::services;
use crate::App;

use super::morphers::IntoApiPostView;

pub async fn list_post_recommendations(
    app: App,
    session_user: SessionUser,
    Query(data): Query<ListPostRecommendations>,
) -> Result<Response, ApiError> {
    let request = services::posts::ListPostRecommendations {
        before: data.after,
        limit: data.limit,
    };

    let response = request
        .perform(&app, &session_user)
        .await?
        .into_iter()
        .map(|view| view.into_api_post_view())
        .collect::<Vec<_>>();

    Ok(Json(response).into_response())
}

pub fn routes() -> Router<App> {
    Router::new().route("/recommendations", get(list_post_recommendations))
}
