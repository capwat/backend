use crate::extract::{LocalInstanceSettings, SessionUser};
use crate::services::util::check_if_admin;
use capwat_error::ApiError;
use capwat_model::InstanceSettings;

pub struct GetSettings;

impl GetSettings {
    #[tracing::instrument(skip(self), name = "services.admin.local_instance.settings")]
    pub async fn perform(
        self,
        user: SessionUser,
        inner: LocalInstanceSettings,
    ) -> Result<InstanceSettings, ApiError> {
        check_if_admin(&user)?;
        Ok((&*inner.0).clone())
    }
}
