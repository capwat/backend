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

use capwat_api_types::post::Post;
use capwat_api_types::user::UserView;
use capwat_model::post::PostView;

fn build_api_post_from_view(view: PostView) -> Post {
    Post {
        id: view.post.id.0,
        created_at: view.post.created.into(),
        last_edited_at: view.post.updated.map(|v| v.into()),
        author: UserView {
            id: view.author.id.0,
            joined_at: view.author.created.into(),
            name: view.author.name,
            display_name: view.author.display_name,
            is_admin: view.author.admin,
            followers: view.author_aggregates.followers as u64,
            following: view.author_aggregates.following as u64,
        },
        content: view.post.content,
    }
}
