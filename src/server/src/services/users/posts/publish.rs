use capwat_error::ApiError;
use capwat_model::post::Post;
use capwat_utils::Sensitive;

use crate::extract::{LocalInstanceSettings, SessionUser};
use crate::services::util::check_post_content;
use crate::App;

#[derive(Debug)]
pub struct PublishPost<'a> {
    pub content: Sensitive<&'a str>,
}

#[derive(Debug)]
pub struct PublishPostResponse {
    pub post: Post,
}

impl PublishPost<'_> {
    #[tracing::instrument(skip_all, fields(self), name = "services.users.profile.post")]
    pub async fn perform(
        self,
        app: &App,
        local_settings: &LocalInstanceSettings,
        session_user: &SessionUser,
    ) -> Result<PublishPostResponse, ApiError> {
        check_post_content(&session_user, &local_settings, &self.content)?;

        todo!()
    }
}
