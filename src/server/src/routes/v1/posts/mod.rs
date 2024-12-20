use axum::extract::Path;
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use capwat_api_types::routes::posts::ListRecommendedPosts;
use capwat_error::ApiError;
use capwat_model::id::PostId;

use crate::extract::Json;
use crate::extract::SessionUser;
use crate::services;
use crate::App;

use super::morphers::IntoApiPostView;

pub fn routes() -> Router<App> {
    Router::new()
        .route("/:id", get(fetch_post))
        .route("/recommended", get(list_post_recommendations))
}

pub async fn fetch_post(app: App, Path(id): Path<PostId>) -> Result<Response, ApiError> {
    let request = services::posts::GetPost { id: id.into() };
    let response = request.perform(&app).await?.into_api_post_view();

    Ok(Json(response).into_response())
}

pub async fn list_post_recommendations(
    app: App,
    session_user: SessionUser,
    Query(data): Query<ListRecommendedPosts>,
) -> Result<Response, ApiError> {
    let request = services::posts::ListRecommendedPost {
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
