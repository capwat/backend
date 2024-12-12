use capwat_error::ApiError;
use capwat_model::post::{InsertPost, Post};
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

        let mut conn = app.db_write().await?;
        let post = InsertPost::builder()
            .author_id(session_user.id)
            .content(&self.content)
            .build()
            .insert(&mut conn)
            .await?;

        Ok(PublishPostResponse { post })
    }
}

#[cfg(test)]
mod tests {
    use crate::extract::LocalInstanceSettings;
    use crate::test_utils::{self, TestResultExt};

    use assert_json_diff::assert_json_include;
    use capwat_model::instance::UpdateInstanceSettings;
    use capwat_utils::Sensitive;
    use serde_json::json;

    #[tracing::instrument]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_reject_if_content_is_empty() {
        let (app, _) = test_utils::build_test_app().await;
        let alice = test_utils::users::start_session()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let mut conn = app.db_write().await.unwrap();
        let local_settings = UpdateInstanceSettings::builder()
            .post_max_characters(5)
            .build()
            .perform_local(&mut conn)
            .await
            .unwrap();

        let local_settings = LocalInstanceSettings::new(local_settings);
        let session_user = alice.get_session_user(&app).await;

        let request = super::PublishPost {
            content: Sensitive::new(""),
        };

        let error = request
            .perform(&app, &local_settings, &session_user)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "publish_post_failed",
                "subcode": "empty_content",
            })
        );
    }

    #[tracing::instrument]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_reject_if_content_reached_max_chars() {
        let (app, _) = test_utils::build_test_app().await;
        let alice = test_utils::users::start_session()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let mut conn = app.db_write().await.unwrap();
        let local_settings = UpdateInstanceSettings::builder()
            .post_max_characters(5)
            .build()
            .perform_local(&mut conn)
            .await
            .unwrap();

        let local_settings = LocalInstanceSettings::new(local_settings);
        let session_user = alice.get_session_user(&app).await;

        let request = super::PublishPost {
            content: Sensitive::new("I'm a weirdo. #weirdo"),
        };

        let error = request
            .perform(&app, &local_settings, &session_user)
            .await
            .expect_error_json();

        assert_json_include!(
            actual: error,
            expected: json!({
                "code": "publish_post_failed",
                "subcode": "too_many_characters",
            })
        );
    }

    #[tracing::instrument]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn should_post_successfully() {
        let (app, _) = test_utils::build_test_app().await;
        let alice = test_utils::users::start_session()
            .app(&app)
            .name("alice")
            .call()
            .await;

        let request = super::PublishPost {
            content: Sensitive::new("I'm a weirdo. #weirdo"),
        };

        let local_settings = test_utils::local_instance::get_settings(&app).await;
        let session_user = alice.get_session_user(&app).await;
        request
            .perform(&app, &local_settings, &session_user)
            .await
            .unwrap();
    }
}
