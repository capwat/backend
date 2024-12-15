use crate::extract::SessionUser;

pub struct LocalProfile;

impl LocalProfile {
    #[tracing::instrument(skip_all, name = "services.users.profile.me")]
    pub async fn perform(self, session_user: SessionUser) -> LocalProfileResponse {
        LocalProfileResponse { session_user }
    }
}

#[must_use]
pub struct LocalProfileResponse {
    pub session_user: SessionUser,
}
