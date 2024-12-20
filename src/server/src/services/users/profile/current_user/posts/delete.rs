use capwat_error::ext::ResultExt;
use capwat_error::{ApiError, ApiErrorCategory};
use capwat_model::id::PostId;
use capwat_model::post::Post;
use capwat_utils::Sensitive;

use crate::extract::SessionUser;
use crate::App;

#[derive(Debug)]
pub struct DeleteCurrentUserPost {
    pub id: Sensitive<PostId>,
}

impl DeleteCurrentUserPost {
    #[tracing::instrument(skip_all, fields(self), name = "services.users.me.publish_post")]
    pub async fn perform(self, app: &App, session_user: &SessionUser) -> Result<(), ApiError> {
        let mut conn = app.db_write().await?;
        let Some(post) = Post::find(&mut conn, *self.id).await? else {
            return Err(ApiError::new(ApiErrorCategory::NotFound).message("Unknown post"));
        };

        if Some(session_user.id) != post.author_id {
            return Err(ApiError::new(ApiErrorCategory::AccessDenied));
        }

        Post::delete(&mut conn, post.id).await?;
        conn.commit().await.erase_context()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::LocalInstanceSettings;
    use crate::services::users::profile::current_user::PublishUserPost;
    use crate::test_utils;

    #[capwat_macros::api_test]
    async fn should_delete_current_user_post(app: App, local_settings: LocalInstanceSettings) {
        let alice = test_utils::users::get_session_data()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let post = PublishUserPost {
            content: Sensitive::new("Hello, World!"),
        }
        .perform(&app, &local_settings, &alice.get_session_user(&app).await)
        .await
        .unwrap()
        .post;

        DeleteCurrentUserPost {
            id: Sensitive::new(post.id),
        }
        .perform(&app, &alice.get_session_user(&app).await)
        .await
        .unwrap();

        // It should delete the content but not the user nor the post
        let deleted_post = Post::find(&mut app.db_read().await.unwrap(), post.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(deleted_post.content, None);
    }
}
