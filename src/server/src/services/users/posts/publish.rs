use capwat_error::ext::ResultExt;
use capwat_error::ApiError;
use capwat_model::post::{InsertPost, Post};
use capwat_utils::Sensitive;

use crate::extract::{LocalInstanceSettings, SessionUser};
use crate::services::util::check_post_content;
use crate::App;

#[derive(Debug)]
pub struct PublishUserPost<'a> {
    pub content: Sensitive<&'a str>,
}

#[derive(Debug)]
pub struct PublishUserPostResponse {
    pub post: Post,
}

impl PublishUserPost<'_> {
    #[tracing::instrument(skip_all, fields(self), name = "services.users.me.publish_post")]
    pub async fn perform(
        self,
        app: &App,
        local_settings: &LocalInstanceSettings,
        session_user: &SessionUser,
    ) -> Result<PublishUserPostResponse, ApiError> {
        check_post_content(&session_user, &local_settings, &self.content)?;

        let mut conn = app.db_write().await?;
        let post = InsertPost::builder()
            .author_id(session_user.id)
            .content(&self.content)
            .build()
            .insert(&mut conn)
            .await?;

        conn.commit().await.erase_context()?;

        Ok(PublishUserPostResponse { post })
    }
}
